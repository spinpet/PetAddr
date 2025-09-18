use PetAddr::pet::address::PetAddress;

fn main() {
    println!("Testing Pet address generation...");
    
    // Test single address generation
    if let Some(address) = PetAddress::generate() {
        println!("Generated Pet address:");
        println!("  Address: {}", address.address);
        println!("  Public Key: {}", address.public_key);
        println!("  Private Key: {}", address.private_key);
        println!("  Ends with 'Pet': {}", address.address.ends_with("Pet"));
    } else {
        println!("Failed to generate Pet address");
    }
}