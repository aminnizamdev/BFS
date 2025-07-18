//! Comprehensive Test Suite for I Protocol Blockchain
//! 
//! This module contains unit tests, stress tests, error handling tests,
//! bug hunting, and edge case testing for the I Protocol blockchain.

use crate::*;
use chrono::Utc;
use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // UNIT TESTS - Core Functionality
    // ============================================================================

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            "test_from".to_string(),
            "test_to".to_string(),
            1000,
            1,
            "test_signature".to_string(),
        );
        
        assert_eq!(tx.from, "test_from");
        assert_eq!(tx.to, "test_to");
        assert_eq!(tx.amount, 1000);
        assert_eq!(tx.fee, TRANSACTION_FEE);
        assert_eq!(tx.nonce, 1);
        assert_eq!(tx.signature, "test_signature");
        assert!(!tx.txn_id.is_empty());
    }

    #[test]
    fn test_transaction_hash_consistency() {
        // Create first transaction
        let mut tx1 = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "sig1".to_string(),
        );
        
        // Create second transaction with same timestamp
        let mut tx2 = Transaction {
            txn_id: String::new(),
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 1000,
            fee: TRANSACTION_FEE,
            nonce: 1,
            timestamp: tx1.timestamp, // Use same timestamp
            signature: "sig1".to_string(),
        };
        
        // Calculate hashes with same timestamp
        tx1.txn_id = tx1.calculate_hash();
        tx2.txn_id = tx2.calculate_hash();
        
        // Same transaction data should produce same hash
        assert_eq!(tx1.calculate_hash(), tx2.calculate_hash());
        assert_eq!(tx1.txn_id, tx2.txn_id);
    }

    #[test]
    fn test_transaction_hash_uniqueness() {
        let tx1 = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "sig1".to_string(),
        );
        
        let tx2 = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1001, // Different amount
            1,
            "sig1".to_string(),
        );
        
        // Different transaction data should produce different hashes
        assert_ne!(tx1.calculate_hash(), tx2.calculate_hash());
        assert_ne!(tx1.txn_id, tx2.txn_id);
    }

    #[test]
    fn test_block_creation() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "signature".to_string(),
        );
        
        let block = Block::new(
            1,
            "parent_hash".to_string(),
            vec![tx],
            4,
        );
        
        assert_eq!(block.header.block_height, 1);
        assert_eq!(block.header.parent_hash, "parent_hash");
        assert_eq!(block.header.difficulty, 4);
        assert_eq!(block.transaction_count, 1);
        assert_eq!(block.transactions.len(), 1);
    }

    #[test]
    fn test_merkle_root_calculation() {
        let tx1 = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "sig1".to_string(),
        );
        
        let tx2 = Transaction::new(
            "charlie".to_string(),
            "diana".to_string(),
            2000,
            1,
            "sig2".to_string(),
        );
        
        let block1 = Block::new(1, "parent".to_string(), vec![tx1.clone()], 4);
        let block2 = Block::new(1, "parent".to_string(), vec![tx1, tx2], 4);
        
        // Different transaction sets should produce different merkle roots
        assert_ne!(block1.header.merkle_root, block2.header.merkle_root);
    }

    #[test]
    fn test_difficulty_target_validation() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "signature".to_string(),
        );
        
        let mut block = Block::new(1, "parent".to_string(), vec![tx], 3);
        
        // Before mining, should not meet difficulty
        assert!(!block.header.meets_difficulty_target());
        
        // After mining, should meet difficulty
        block.mine_block();
        assert!(block.header.meets_difficulty_target());
    }

    // ============================================================================
    // EDGE CASE TESTS
    // ============================================================================

    #[test]
    fn test_empty_transaction_list() {
        let block = Block::new(
            1,
            "parent_hash".to_string(),
            vec![], // Empty transaction list
            4,
        );
        
        assert_eq!(block.transaction_count, 0);
        assert_eq!(block.transactions.len(), 0);
        assert!(!block.header.merkle_root.is_empty()); // Should still have a merkle root
    }

    #[test]
    fn test_zero_amount_transaction() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            0, // Zero amount
            1,
            "signature".to_string(),
        );
        
        assert_eq!(tx.amount, 0);
        assert_eq!(tx.fee, TRANSACTION_FEE); // Fee should still be applied
        assert!(!tx.txn_id.is_empty()); // Should still generate valid hash
    }

    #[test]
    fn test_maximum_amount_transaction() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            u64::MAX, // Maximum possible amount
            1,
            "signature".to_string(),
        );
        
        assert_eq!(tx.amount, u64::MAX);
        assert!(!tx.txn_id.is_empty());
    }

    #[test]
    fn test_empty_string_fields() {
        let tx = Transaction::new(
            "".to_string(), // Empty from
            "".to_string(), // Empty to
            1000,
            1,
            "".to_string(), // Empty signature
        );
        
        assert_eq!(tx.from, "");
        assert_eq!(tx.to, "");
        assert_eq!(tx.signature, "");
        assert!(!tx.txn_id.is_empty()); // Should still generate hash
    }

    #[test]
    fn test_very_long_string_fields() {
        let long_string = "a".repeat(10000); // Very long string
        
        let tx = Transaction::new(
            long_string.clone(),
            long_string.clone(),
            1000,
            1,
            long_string.clone(),
        );
        
        assert_eq!(tx.from.len(), 10000);
        assert_eq!(tx.to.len(), 10000);
        assert_eq!(tx.signature.len(), 10000);
        assert!(!tx.txn_id.is_empty());
    }

    #[test]
    fn test_zero_difficulty_mining() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "signature".to_string(),
        );
        
        let mut block = Block::new(1, "parent".to_string(), vec![tx], 0);
        let hash = block.mine_block();
        
        // With zero difficulty, any hash should be valid
        assert!(block.header.meets_difficulty_target());
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_high_difficulty_mining() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "signature".to_string(),
        );
        
        let mut block = Block::new(1, "parent".to_string(), vec![tx], 6); // High difficulty
        let start_time = Utc::now();
        let hash = block.mine_block();
        let end_time = Utc::now();
        
        assert!(block.header.meets_difficulty_target());
        assert!(hash.starts_with("000000")); // Should have 6 leading zeros
        
        // Should take some time to mine
        let duration = end_time - start_time;
        println!("High difficulty mining took: {} ms", duration.num_milliseconds());
    }

    // ============================================================================
    // STRESS TESTS
    // ============================================================================

    #[test]
    fn test_large_transaction_block() {
        let mut transactions = Vec::new();
        
        // Create 1000 transactions
        for i in 0..1000 {
            let tx = Transaction::new(
                format!("sender_{}", i),
                format!("recipient_{}", i),
                1000 + i as u64,
                i as u64 + 1,
                format!("signature_{}", i),
            );
            transactions.push(tx);
        }
        
        let start_time = Utc::now();
        let block = Block::new(1, "parent".to_string(), transactions, 3);
        let creation_time = Utc::now() - start_time;
        
        assert_eq!(block.transaction_count, 1000);
        assert_eq!(block.transactions.len(), 1000);
        assert!(!block.header.merkle_root.is_empty());
        
        println!("Large block creation took: {} ms", creation_time.num_milliseconds());
    }

    #[test]
    fn test_hash_collision_resistance() {
        let mut hashes = HashSet::new();
        
        // Generate 10000 different transactions and check for hash collisions
        for i in 0..10000 {
            let tx = Transaction::new(
                format!("sender_{}", i),
                format!("recipient_{}", i % 100), // Some overlap in recipients
                1000 + (i % 1000) as u64, // Some overlap in amounts
                (i % 10) as u64 + 1, // Some overlap in nonces
                format!("signature_{}", i),
            );
            
            let hash = tx.calculate_hash();
            assert!(!hashes.contains(&hash), "Hash collision detected for transaction {}", i);
            hashes.insert(hash);
        }
        
        println!("Generated {} unique hashes without collisions", hashes.len());
    }

    #[test]
    fn test_serialization_performance() {
        let mut transactions = Vec::new();
        
        // Create 100 transactions
        for i in 0..100 {
            let tx = Transaction::new(
                format!("sender_{}", i),
                format!("recipient_{}", i),
                1000 + i as u64,
                i as u64 + 1,
                format!("signature_{}", i),
            );
            transactions.push(tx);
        }
        
        let block = Block::new(1, "parent".to_string(), transactions, 3);
        
        let start_time = Utc::now();
        let json = serde_json::to_string(&block).expect("Serialization should succeed");
        let serialization_time = Utc::now() - start_time;
        
        let start_time = Utc::now();
        let _deserialized: Block = serde_json::from_str(&json).expect("Deserialization should succeed");
        let deserialization_time = Utc::now() - start_time;
        
        println!("Serialization took: {} ms", serialization_time.num_milliseconds());
        println!("Deserialization took: {} ms", deserialization_time.num_milliseconds());
        println!("JSON size: {} bytes", json.len());
        
        assert!(!json.is_empty());
    }

    // ============================================================================
    // ERROR HANDLING TESTS
    // ============================================================================

    #[test]
    fn test_invalid_json_deserialization() {
        let invalid_json = "{\"invalid\": \"json\", \"missing_fields\": true}";
        
        let result: Result<Block, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "Should fail to deserialize invalid JSON");
    }

    #[test]
    fn test_malformed_json_deserialization() {
        let malformed_json = "{invalid json syntax";
        
        let result: Result<Block, _> = serde_json::from_str(malformed_json);
        assert!(result.is_err(), "Should fail to deserialize malformed JSON");
    }

    #[test]
    fn test_unicode_handling() {
        let tx = Transaction::new(
            "🚀 Unicode sender 中文".to_string(),
            "🎯 Unicode recipient العربية".to_string(),
            1000,
            1,
            "🔐 Unicode signature русский".to_string(),
        );
        
        assert!(!tx.txn_id.is_empty());
        
        // Test serialization with Unicode
        let json = serde_json::to_string(&tx).expect("Should serialize Unicode correctly");
        let deserialized: Transaction = serde_json::from_str(&json).expect("Should deserialize Unicode correctly");
        
        assert_eq!(tx.from, deserialized.from);
        assert_eq!(tx.to, deserialized.to);
        assert_eq!(tx.signature, deserialized.signature);
    }

    // ============================================================================
    // BLOCKCHAIN STRUCT TESTS
    // ============================================================================

    #[test]
    fn test_blockchain_initialization() {
        let blockchain = crate::Blockchain::new(4);
        
        // Check initial state
        let (chain_length, pending_count, difficulty) = blockchain.get_stats();
        assert_eq!(chain_length, 1); // Should have genesis block
        assert_eq!(pending_count, 0); // No pending transactions
        assert_eq!(difficulty, 4); // Correct difficulty
        
        // Check genesis block
        let genesis = blockchain.get_latest_block().unwrap();
        assert_eq!(genesis.header.block_height, 0);
        assert_eq!(genesis.header.parent_hash, "0000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(genesis.transactions.len(), 0);
        assert!(genesis.header.meets_difficulty_target());
    }

    #[test]
    fn test_blockchain_add_transaction() {
        let mut blockchain = crate::Blockchain::new(3);
        
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "signature".to_string(),
        );
        
        blockchain.add_transaction(tx.clone());
        
        let (_, pending_count, _) = blockchain.get_stats();
        assert_eq!(pending_count, 1);
        assert_eq!(blockchain.pending_transactions.len(), 1);
        assert_eq!(blockchain.pending_transactions[0].txn_id, tx.txn_id);
    }

    #[test]
    fn test_blockchain_mine_pending_transactions() {
        let mut blockchain = crate::Blockchain::new(2); // Lower difficulty for faster test
        
        // Add some transactions
        let tx1 = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "sig1".to_string(),
        );
        
        let tx2 = Transaction::new(
            "charlie".to_string(),
            "diana".to_string(),
            2000,
            1,
            "sig2".to_string(),
        );
        
        blockchain.add_transaction(tx1);
        blockchain.add_transaction(tx2);
        
        // Mine the transactions
        let result = blockchain.mine_pending_transactions();
        assert!(result.is_ok());
        
        // Check state after mining
        let (chain_length, pending_count, _) = blockchain.get_stats();
        assert_eq!(chain_length, 2); // Genesis + 1 new block
        assert_eq!(pending_count, 0); // Pending transactions should be cleared
        
        // Check the new block
        let latest_block = blockchain.get_latest_block().unwrap();
        assert_eq!(latest_block.header.block_height, 1);
        assert_eq!(latest_block.transactions.len(), 2);
        assert!(latest_block.header.meets_difficulty_target());
    }

    #[test]
    fn test_blockchain_mine_empty_pending() {
        let mut blockchain = crate::Blockchain::new(3);
        
        // Try to mine with no pending transactions
        let result = blockchain.mine_pending_transactions();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No pending transactions to mine");
        
        // Chain should remain unchanged
        let (chain_length, pending_count, _) = blockchain.get_stats();
        assert_eq!(chain_length, 1); // Still just genesis
        assert_eq!(pending_count, 0);
    }

    #[test]
    fn test_blockchain_chain_validation() {
        let mut blockchain = crate::Blockchain::new(2);
        
        // Add and mine several blocks
        for i in 0..3 {
            let tx = Transaction::new(
                format!("sender_{}", i),
                format!("recipient_{}", i),
                1000 + i as u64,
                1,
                format!("signature_{}", i),
            );
            
            blockchain.add_transaction(tx);
            let result = blockchain.mine_pending_transactions();
            assert!(result.is_ok());
        }
        
        // Chain should be valid
        assert!(blockchain.is_chain_valid());
        
        // Check final state
        let (chain_length, pending_count, _) = blockchain.get_stats();
        assert_eq!(chain_length, 4); // Genesis + 3 mined blocks
        assert_eq!(pending_count, 0);
    }

    #[test]
    fn test_blockchain_multiple_difficulties() {
        let difficulties = vec![1, 2, 3, 4, 5];
        
        for difficulty in difficulties {
            let mut blockchain = crate::Blockchain::new(difficulty);
            
            let tx = Transaction::new(
                "test_sender".to_string(),
                "test_recipient".to_string(),
                1000,
                1,
                "test_signature".to_string(),
            );
            
            blockchain.add_transaction(tx);
            
            let start_time = Utc::now();
            let result = blockchain.mine_pending_transactions();
            let end_time = Utc::now();
            
            assert!(result.is_ok());
            
            let latest_block = blockchain.get_latest_block().unwrap();
            assert!(latest_block.header.meets_difficulty_target());
            
            let duration = end_time - start_time;
            println!("Difficulty {}: Mining took {} ms", difficulty, duration.num_milliseconds());
        }
    }

    #[test]
    fn test_blockchain_large_transaction_volume() {
        let mut blockchain = crate::Blockchain::new(2); // Lower difficulty for speed
        
        // Add 100 transactions
        for i in 0..100 {
            let tx = Transaction::new(
                format!("sender_{}", i),
                format!("recipient_{}", i % 10), // Some recipients get multiple transactions
                1000 + i as u64,
                (i % 5) as u64 + 1,
                format!("signature_{}", i),
            );
            blockchain.add_transaction(tx);
        }
        
        // Mine all transactions
        let result = blockchain.mine_pending_transactions();
        assert!(result.is_ok());
        
        // Verify final state
        let (chain_length, pending_count, _) = blockchain.get_stats();
        assert_eq!(chain_length, 2); // Genesis + 1 large block
        assert_eq!(pending_count, 0);
        
        let latest_block = blockchain.get_latest_block().unwrap();
        assert_eq!(latest_block.transactions.len(), 100);
        assert!(latest_block.header.meets_difficulty_target());
        assert!(blockchain.is_chain_valid());
    }

    #[test]
    fn test_blockchain_parent_hash_consistency() {
        let mut blockchain = crate::Blockchain::new(2);
        
        let mut previous_hash = blockchain.get_latest_block().unwrap().calculate_hash();
        
        // Mine 5 blocks and verify parent hash consistency
        for i in 0..5 {
            let tx = Transaction::new(
                format!("sender_{}", i),
                format!("recipient_{}", i),
                1000 + i as u64,
                1,
                format!("signature_{}", i),
            );
            
            blockchain.add_transaction(tx);
            let result = blockchain.mine_pending_transactions();
            assert!(result.is_ok());
            
            let latest_block = blockchain.get_latest_block().unwrap();
            assert_eq!(latest_block.header.parent_hash, previous_hash);
            
            previous_hash = latest_block.calculate_hash();
        }
        
        assert!(blockchain.is_chain_valid());
    }

    // ============================================================================
    // BLOCKCHAIN INTEGRITY TESTS
    // ============================================================================

    #[test]
    fn test_block_chain_integrity() {
        // Create genesis block
        let genesis_tx = Transaction::new(
            "genesis".to_string(),
            "initial_holder".to_string(),
            1000000000,
            1,
            "genesis_signature".to_string(),
        );
        
        let mut genesis_block = Block::new(
            0,
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            vec![genesis_tx],
            3,
        );
        
        let genesis_hash = genesis_block.mine_block();
        
        // Create second block
        let tx1 = Transaction::new(
            "initial_holder".to_string(),
            "alice".to_string(),
            500000000,
            1,
            "transfer_signature".to_string(),
        );
        
        let mut block1 = Block::new(
            1,
            genesis_hash.clone(),
            vec![tx1],
            3,
        );
        
        let block1_hash = block1.mine_block();
        
        // Verify chain integrity
        assert_eq!(genesis_block.header.block_height, 0);
        assert_eq!(block1.header.block_height, 1);
        assert_eq!(block1.header.parent_hash, genesis_hash);
        assert!(genesis_block.header.meets_difficulty_target());
        assert!(block1.header.meets_difficulty_target());
        
        println!("Genesis hash: {}", genesis_hash);
        println!("Block 1 hash: {}", block1_hash);
        println!("Block 1 parent: {}", block1.header.parent_hash);
    }

    #[test]
    fn test_nonce_overflow_handling() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
            "signature".to_string(),
        );
        
        let mut block = Block::new(1, "parent".to_string(), vec![tx], 4);
        
        // Set nonce to near maximum to test overflow handling
        block.header.nonce = u64::MAX - 100;
        
        let hash = block.mine_block();
        
        // Should still find a valid hash (or handle overflow gracefully)
        assert!(!hash.is_empty());
        println!("Final nonce after near-overflow: {}", block.header.nonce);
    }

    // ============================================================================
    // PERFORMANCE BENCHMARKS
    // ============================================================================

    #[test]
    fn benchmark_hash_performance() {
        let tx = Transaction::new(
            "performance_test_sender".to_string(),
            "performance_test_recipient".to_string(),
            1000,
            1,
            "performance_test_signature".to_string(),
        );
        
        let iterations = 10000;
        let start_time = Utc::now();
        
        for _ in 0..iterations {
            let _ = tx.calculate_hash();
        }
        
        let end_time = Utc::now();
        let duration = end_time - start_time;
        let hashes_per_second = iterations as f64 / duration.num_milliseconds() as f64 * 1000.0;
        
        println!("Hash performance: {:.0} hashes/second", hashes_per_second);
        assert!(hashes_per_second > 1000.0, "Hash performance should be > 1000 hashes/second");
    }

    #[test]
    fn benchmark_mining_performance() {
        let tx = Transaction::new(
            "mining_test_sender".to_string(),
            "mining_test_recipient".to_string(),
            1000,
            1,
            "mining_test_signature".to_string(),
        );
        
        let difficulties = vec![1, 2, 3, 4];
        
        for difficulty in difficulties {
            let mut block = Block::new(1, "parent".to_string(), vec![tx.clone()], difficulty);
            
            let start_time = Utc::now();
            let hash = block.mine_block();
            let end_time = Utc::now();
            
            let duration = end_time - start_time;
            let expected_zeros = "0".repeat(difficulty as usize);
            
            assert!(hash.starts_with(&expected_zeros));
            println!("Difficulty {}: {} ms, nonce: {}, hash: {}", 
                     difficulty, duration.num_milliseconds(), block.header.nonce, hash);
        }
    }
}