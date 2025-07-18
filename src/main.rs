use i_protocol::{Blockchain, TRANSACTION_FEE};

fn main() {
    println!("=== I Protocol Blockchain Node ===");
    println!("CEO: Amin Nizam");
    println!("Senior Director of Development: Grey");
    println!("Language: Rust");
    println!("Philosophy: Practical, Performance-focused, Light but Powerful\n");
    
    // Initialize blockchain with difficulty 4
    let blockchain = Blockchain::new(4);
    
    // Display genesis block in separate terminal
    if let Some(genesis) = blockchain.get_latest_block() {
        blockchain.display_block_in_terminal(genesis);
    }
    
    println!("[INIT] I Protocol blockchain initialized");
    println!("   Genesis block created");
    println!("   Difficulty: {} (require {} leading zeros)", blockchain.get_difficulty(), blockchain.get_difficulty());
    println!("   Transaction fee: {} units (0.001 I tokens)", TRANSACTION_FEE);
    
    println!("\n[STATUS] Blockchain Status:");
    println!("   Chain length: {} blocks", blockchain.chain_length());
    println!("   Pending transactions: {}", blockchain.pending_count());
    println!("   Mining difficulty: {}", blockchain.get_difficulty());
    println!("   Chain valid: {}", blockchain.is_chain_valid());
    
    if let Some(latest_block) = blockchain.get_latest_block() {
        println!("   Latest block hash: {}", latest_block.calculate_hash());
        println!("   Latest block height: {}", latest_block.header.block_height);
    }
    
    println!("\n[READY] I Protocol blockchain node is ready for transactions");
    println!("\nNext steps:");
    println!("   1. Implement transaction creation API");
    println!("   2. Add Ed25519 signature verification");
    println!("   3. Implement network layer for peer communication");
    println!("   4. Add wallet functionality");
    println!("   5. Create REST API endpoints");
    
    println!("\n[INFO] To add transactions and mine blocks, use the blockchain API (to be implemented)");
    println!("[INFO] Current state: Clean blockchain with genesis block only");
}