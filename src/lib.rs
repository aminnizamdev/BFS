use blake3::Hasher;
use chrono::{DateTime, Utc};
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::VecDeque;

/// Fixed transaction fee for MVP (0.001 I tokens)
pub const TRANSACTION_FEE: u64 = 1_000_000; // Using satoshi-like precision (1 I = 100_000_000 units)

/// Transaction structure representing value transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction ID (Blake3 hash of transaction data)
    pub txn_id: String,
    /// Sender's Ed25519 public key (32 bytes)
    pub from: String,
    /// Recipient's address derived from public key
    pub to: String,
    /// Amount to transfer (in smallest units, like satoshis)
    pub amount: u64,
    /// Transaction fee (fixed at 0.001 I for MVP)
    pub fee: u64,
    /// Account nonce for replay protection
    pub nonce: u64,
    /// Transaction creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Ed25519 signature (64 bytes)
    pub signature: String,
}

/// Block header containing metadata and PoW solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block number in the chain (0 = genesis block)
    pub block_height: u64,
    /// Hash of the previous block
    pub parent_hash: String,
    /// Merkle root of all transactions in this block
    pub merkle_root: String,
    /// Block creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Proof of Work difficulty target
    pub difficulty: u32,
    /// Proof of Work nonce solution
    pub nonce: u64,
}

/// Complete block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header with PoW solution
    pub header: BlockHeader,
    /// Number of transactions in this block
    pub transaction_count: u32,
    /// List of transactions
    pub transactions: Vec<Transaction>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        from: String,
        to: String,
        amount: u64,
        nonce: u64,
        signature: String,
    ) -> Self {
        let timestamp = Utc::now();
        let mut tx = Transaction {
            txn_id: String::new(),
            from,
            to,
            amount,
            fee: TRANSACTION_FEE,
            nonce,
            timestamp,
            signature,
        };
        
        // Calculate transaction ID
        tx.txn_id = tx.calculate_hash();
        tx
    }

    /// Calculate Blake3 hash of transaction data
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Hasher::new();
        
        // Hash all fields except txn_id and signature
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(&self.amount.to_le_bytes());
        hasher.update(&self.fee.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        
        hex::encode(hasher.finalize().as_bytes())
    }

    /// Verify the Ed25519 signature of this transaction
    pub fn verify_signature(&self, public_key: &VerifyingKey) -> Result<bool, Box<dyn std::error::Error>> {
        let message = self.get_signing_message();
        let signature_bytes = hex::decode(&self.signature)?;
        
        // Convert Vec<u8> to [u8; 64]
        if signature_bytes.len() != 64 {
            return Ok(false);
        }
        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&signature_bytes);
        
        let signature = Signature::from_bytes(&sig_array);
        
        match public_key.verify(message.as_bytes(), &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get the message that should be signed
    fn get_signing_message(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            self.from, self.to, self.amount, self.fee, self.nonce, self.timestamp.to_rfc3339()
        )
    }
}

impl BlockHeader {
    /// Create a new block header
    pub fn new(
        block_height: u64,
        parent_hash: String,
        merkle_root: String,
        difficulty: u32,
    ) -> Self {
        BlockHeader {
            block_height,
            parent_hash,
            merkle_root,
            timestamp: Utc::now(),
            difficulty,
            nonce: 0,
        }
    }

    /// Calculate Blake3 hash of block header
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Hasher::new();
        
        hasher.update(&self.block_height.to_le_bytes());
        hasher.update(self.parent_hash.as_bytes());
        hasher.update(self.merkle_root.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(&self.difficulty.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        
        hex::encode(hasher.finalize().as_bytes())
    }

    /// Check if the block header meets the difficulty target
    pub fn meets_difficulty_target(&self) -> bool {
        let hash = self.calculate_hash();
        let required_zeros = self.difficulty;
        
        // Count leading zeros in hex representation
        let leading_zeros = hash.chars()
            .take_while(|&c| c == '0')
            .count() as u32;
            
        leading_zeros >= required_zeros
    }
}

impl Block {
    /// Create a new block
    pub fn new(
        block_height: u64,
        parent_hash: String,
        transactions: Vec<Transaction>,
        difficulty: u32,
    ) -> Self {
        let merkle_root = Self::calculate_merkle_root(&transactions);
        let header = BlockHeader::new(block_height, parent_hash, merkle_root, difficulty);
        let transaction_count = transactions.len() as u32;
        
        Block {
            header,
            transaction_count,
            transactions,
        }
    }

    /// Calculate the Blake3 hash of this block
    pub fn calculate_hash(&self) -> String {
        self.header.calculate_hash()
    }

    /// Calculate merkle root of transactions (simplified version for MVP)
    fn calculate_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "0".repeat(64); // Empty merkle root
        }

        let mut hasher = Hasher::new();
        for tx in transactions {
            hasher.update(tx.txn_id.as_bytes());
        }
        
        hex::encode(hasher.finalize().as_bytes())
    }

    /// Mine this block by finding a valid nonce
    pub fn mine_block(&mut self) -> String {
        println!("Mining block at height {}...", self.header.block_height);
        
        loop {
            if self.header.meets_difficulty_target() {
                let block_hash = self.calculate_hash();
                println!("Block mined! Hash: {}", block_hash);
                println!("Nonce: {}", self.header.nonce);
                return block_hash;
            }
            
            // Handle nonce overflow by resetting timestamp and nonce
            if self.header.nonce == u64::MAX {
                println!("Nonce overflow detected, updating timestamp and resetting nonce");
                self.header.timestamp = Utc::now();
                self.header.nonce = 0;
            } else {
                self.header.nonce += 1;
            }
            
            // Progress indicator every 100K attempts
            if self.header.nonce % 100_000 == 0 {
                println!("Mining... nonce: {}", self.header.nonce);
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction {{ ID: {}, From: {}..., To: {}..., Amount: {}, Fee: {}, Nonce: {} }}",
            &self.txn_id[..8],
            &self.from[..8],
            &self.to[..8],
            self.amount,
            self.fee,
            self.nonce
        )
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block {{ Height: {}, Hash: {}, Transactions: {}, Difficulty: {}, Nonce: {} }}",
            self.header.block_height,
            &self.calculate_hash()[..16],
            self.transaction_count,
            self.header.difficulty,
            self.header.nonce
        )
    }
}

/// Simple blockchain structure to hold the chain state
#[derive(Debug, Clone)]
pub struct Blockchain {
    chain: VecDeque<Block>,
    pending_transactions: Vec<Transaction>,
    difficulty: u32,
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new(difficulty: u32) -> Self {
        let mut blockchain = Blockchain {
            chain: VecDeque::new(),
            pending_transactions: Vec::new(),
            difficulty,
        };
        
        // Create genesis block
        let mut genesis_block = Block::new(
            0,
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            Vec::new(), // Genesis block has no transactions
            difficulty,
        );
        
        // Mine the genesis block to meet difficulty target
        genesis_block.mine_block();
        
        blockchain.chain.push_back(genesis_block);
        blockchain
    }
    
    /// Get the latest block in the chain
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.back()
    }
    
    /// Add a transaction to the pending pool
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }
    
    /// Mine pending transactions into a new block
    pub fn mine_pending_transactions(&mut self) -> Result<String, String> {
        if self.pending_transactions.is_empty() {
            return Err("No pending transactions to mine".to_string());
        }
        
        let latest_block = self.get_latest_block()
            .ok_or("No blocks in chain")?;
        
        let new_height = latest_block.header.block_height + 1;
        let parent_hash = latest_block.calculate_hash();
        
        let mut new_block = Block::new(
            new_height,
            parent_hash,
            self.pending_transactions.clone(),
            self.difficulty,
        );
        
        let block_hash = new_block.mine_block();
        
        self.chain.push_back(new_block);
        self.pending_transactions.clear();
        
        Ok(block_hash)
    }
    
    /// Get blockchain statistics
    pub fn get_stats(&self) -> (usize, usize, u32) {
        (self.chain.len(), self.pending_transactions.len(), self.difficulty)
    }
    
    /// Validate the entire blockchain
    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];
            
            // Check if current block's parent hash matches previous block's hash
            if current_block.header.parent_hash != previous_block.calculate_hash() {
                return false;
            }
            
            // Check if current block meets difficulty target
            if !current_block.header.meets_difficulty_target() {
                return false;
            }
        }
        true
    }
    
    /// Get the chain length
    pub fn chain_length(&self) -> usize {
        self.chain.len()
    }
    
    /// Get pending transactions count
    pub fn pending_count(&self) -> usize {
        self.pending_transactions.len()
    }
    
    /// Get mining difficulty
    pub fn get_difficulty(&self) -> u32 {
        self.difficulty
    }
    
    /// Display a block's full details in a separate terminal window
    pub fn display_block_in_terminal(&self, block: &Block) {
        let mut full_block_display = String::new();
        full_block_display.push_str(&format!("{}\n", "=".repeat(80)));
        full_block_display.push_str(&format!("                    BLOCK #{} DISPLAY\n", block.header.block_height));
        full_block_display.push_str(&format!("{}\n", "=".repeat(80)));
        
        full_block_display.push_str("\nBLOCK HEADER:\n");
        full_block_display.push_str(&format!("   Block Height: {}\n", block.header.block_height));
        full_block_display.push_str(&format!("   Parent Hash: {}\n", block.header.parent_hash));
        full_block_display.push_str(&format!("   Merkle Root: {}\n", block.header.merkle_root));
        full_block_display.push_str(&format!("   Timestamp: {}\n", block.header.timestamp));
        full_block_display.push_str(&format!("   Difficulty: {}\n", block.header.difficulty));
        full_block_display.push_str(&format!("   Nonce: {}\n", block.header.nonce));
        full_block_display.push_str(&format!("   Block Hash: {}\n", block.calculate_hash()));
        
        if block.transactions.is_empty() {
            full_block_display.push_str("\nTRANSACTIONS:\n");
            full_block_display.push_str("   [GENESIS BLOCK - No transactions]\n");
        } else {
            full_block_display.push_str("\nTRANSACTION DETAILS:\n");
            for (i, tx) in block.transactions.iter().enumerate() {
                full_block_display.push_str(&format!("   \n   Transaction #{}\n", i + 1));
                full_block_display.push_str(&format!("   - ID: {}\n", tx.txn_id));
                full_block_display.push_str(&format!("   - From: {}\n", tx.from));
                full_block_display.push_str(&format!("   - To: {}\n", tx.to));
                full_block_display.push_str(&format!("   - Amount: {} units ({:.3} I tokens)\n", tx.amount, tx.amount as f64 / 1_000_000_000.0));
                full_block_display.push_str(&format!("   - Fee: {} units ({:.3} I tokens)\n", tx.fee, tx.fee as f64 / 1_000_000_000.0));
                full_block_display.push_str(&format!("   - Nonce: {}\n", tx.nonce));
                full_block_display.push_str(&format!("   - Timestamp: {}\n", tx.timestamp));
                full_block_display.push_str(&format!("   - Signature: {}\n", tx.signature));
                full_block_display.push_str(&format!("   - Hash: {}\n", tx.calculate_hash()));
            }
        }
        
        full_block_display.push_str("\nBLOCK STATISTICS:\n");
        full_block_display.push_str(&format!("   Total Transactions: {}\n", block.transactions.len()));
        let total_amount: u64 = block.transactions.iter().map(|tx| tx.amount).sum();
        let total_fees: u64 = block.transactions.iter().map(|tx| tx.fee).sum();
        full_block_display.push_str(&format!("   Total Amount Transferred: {} units ({:.3} I tokens)\n", total_amount, total_amount as f64 / 1_000_000_000.0));
        full_block_display.push_str(&format!("   Total Fees Collected: {} units ({:.3} I tokens)\n", total_fees, total_fees as f64 / 1_000_000_000.0));
        full_block_display.push_str(&format!("   Block Size (JSON): {} bytes\n", serde_json::to_string(block).unwrap_or_default().len()));
        
        full_block_display.push_str("\nSERIALIZED BLOCK (JSON):\n");
        match serde_json::to_string_pretty(block) {
            Ok(json) => {
                full_block_display.push_str(&format!("{}\n", json));
            }
            Err(e) => full_block_display.push_str(&format!("   [ERROR] Failed to serialize: {}\n", e)),
        }
        
        full_block_display.push_str(&format!("\n{}\n", "=".repeat(80)));
        full_block_display.push_str(&format!("                   END BLOCK #{} DISPLAY\n", block.header.block_height));
        full_block_display.push_str(&format!("{}\n", "=".repeat(80)));
        
        // Write to file and open in new terminal
        let filename = format!("block_{}_display.txt", block.header.block_height);
        match std::fs::write(&filename, &full_block_display) {
            Ok(_) => {
                println!("[INFO] Block #{} display saved to {}", block.header.block_height, filename);
                println!("[INFO] Opening block #{} display in separate terminal...", block.header.block_height);
                
                // Open new CMD window to display the block
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "cmd", "/k", &format!("type {} && echo. && echo Press any key to close this window... && pause >nul", filename)])
                    .spawn();
            }
            Err(e) => println!("[ERROR] Failed to write block display file: {}", e),
        }
    }
}

// Include tests module
#[cfg(test)]
mod tests;