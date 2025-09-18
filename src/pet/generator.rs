use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

use crate::config::PetGeneratorConfig;
use super::address::PetAddress;
use super::storage::PetStorage;

pub struct PetGenerator {
    storage: Arc<PetStorage>,
    config: PetGeneratorConfig,
    is_running: Arc<Mutex<bool>>,
}

impl PetGenerator {
    pub fn new(storage: Arc<PetStorage>, config: PetGeneratorConfig) -> Self {
        Self {
            storage,
            config,
            is_running: Arc::new(Mutex::new(false)),
        }
    }
    
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.is_running.lock().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        info!("Starting Pet address generator");
        
        let storage = Arc::clone(&self.storage);
        let config = self.config.clone();
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            loop {
                {
                    let running = is_running.lock().await;
                    if !*running {
                        break;
                    }
                }
                
                match storage.count_addresses() {
                    Ok(count) => {
                        if count < config.pool_size {
                            let need_to_generate = config.pool_size - count;
                            let batch_size = std::cmp::min(need_to_generate, config.batch_size);
                            
                            info!("Current address count: {}, generating {} more addresses", count, batch_size);
                            
                            Self::generate_batch(&storage, batch_size).await;
                        }
                    }
                    Err(e) => {
                        error!("Failed to check address count: {}", e);
                    }
                }
                
                sleep(Duration::from_secs(5)).await;
            }
            
            info!("Pet address generator stopped");
        });
        
        Ok(())
    }
    
    pub async fn stop(&self) {
        let mut running = self.is_running.lock().await;
        *running = false;
        info!("Stopping Pet address generator");
    }
    
    async fn generate_batch(storage: &PetStorage, count: usize) {
        let (tx, mut rx) = mpsc::channel(count);
        
        // Spawn generation tasks
        for _ in 0..count {
            let tx = tx.clone();
            tokio::spawn(async move {
                if let Some(address) = PetAddress::generate() {
                    if tx.send(address).await.is_err() {
                        warn!("Failed to send generated address to channel");
                    }
                }
            });
        }
        
        drop(tx); // Close the sender
        
        // Collect generated addresses
        let mut generated_count = 0;
        while let Some(address) = rx.recv().await {
            match storage.store_address(address) {
                Ok(id) => {
                    generated_count += 1;
                    info!("Generated Pet address with ID: {}", id);
                }
                Err(e) => {
                    error!("Failed to store Pet address: {}", e);
                }
            }
        }
        
        info!("Generated {} Pet addresses in batch", generated_count);
    }
    
    pub async fn get_current_count(&self) -> Result<usize> {
        self.storage.count_addresses()
    }
}