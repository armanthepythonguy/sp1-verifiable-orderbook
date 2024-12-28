use sha3::{Digest, Keccak256};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct TreeNode {
    key: Vec<u8>,
    value: u128,
    hash: Vec<u8>,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

#[derive(Debug)]
pub struct Balance {
    pub user_address: Vec<u8>,
    pub token_address: Vec<u8>,
    pub balance: u128,
}

pub struct OrderbookMerkleTree {
    root: Option<TreeNode>,
    leaves: HashMap<Vec<u8>, TreeNode>,
}

impl OrderbookMerkleTree {
    pub fn new() -> Self {
        Self {
            root: None,
            leaves: HashMap::new(),
        }
    }

    fn create_key(&self, user_address: &[u8], token_address: &[u8]) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        hasher.update(user_address);
        hasher.update(token_address);
        hasher.finalize().to_vec()
    }

    fn hash_node(&self, node: &TreeNode) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        hasher.update(&node.key);
        hasher.update(&node.value.to_be_bytes());
        hasher.finalize().to_vec()
    }

    pub fn update_balance(&mut self, user_address: &[u8], token_address: &[u8], balance: u128) {
        let key = self.create_key(user_address, token_address);
        
        let mut node = TreeNode {
            key: key.clone(),
            value: balance,
            hash: Vec::new(),
            left: None,
            right: None,
        };
        node.hash = self.hash_node(&node);
        
        self.leaves.insert(key, node);
        self.rebuild_tree();
    }

    pub fn batch_update(&mut self, updates: &[Balance]) {
        for update in updates {
            let key = self.create_key(&update.user_address, &update.token_address);
            let mut node = TreeNode {
                key: key.clone(),
                value: update.balance,
                hash: Vec::new(),
                left: None,
                right: None,
            };
            node.hash = self.hash_node(&node);
            self.leaves.insert(key, node);
        }
        self.rebuild_tree();
    }

    fn rebuild_tree(&mut self) {
        let mut nodes: Vec<TreeNode> = self.leaves.values().cloned().collect();
        nodes.sort_by(|a, b| a.key.cmp(&b.key)); // Sort nodes by key for deterministic tree structure
        
        if nodes.is_empty() {
            self.root = None;
            return;
        }

        while nodes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in nodes.chunks(2) {
                let left = &chunk[0];
                let right = chunk.get(1).unwrap_or(left);

                let mut hasher = Keccak256::new();
                hasher.update(&left.hash);
                hasher.update(&right.hash);
                let parent_hash = hasher.finalize().to_vec();

                let parent = TreeNode {
                    key: Vec::new(),
                    value: 0,
                    hash: parent_hash,
                    left: Some(Box::new(left.clone())),
                    right: if chunk.len() > 1 { 
                        Some(Box::new(right.clone())) 
                    } else { 
                        None 
                    },
                };

                next_level.push(parent);
            }

            nodes = next_level;
        }

        self.root = Some(nodes.remove(0));
    }

    pub fn get_root(&self) -> Vec<u8> {
        match &self.root {
            Some(root) => root.hash.clone(),
            None => vec![0; 32],
        }
    }

    fn collect_proof(&self, target_key: &[u8]) -> Vec<Vec<u8>> {
        let mut proof = Vec::new();
        let mut nodes: Vec<TreeNode> = self.leaves.values().cloned().collect();
        nodes.sort_by(|a, b| a.key.cmp(&b.key)); // Sort nodes by key
        
        if nodes.is_empty() {
            return proof;
        }

        let mut target_idx = nodes.iter().position(|node| node.key == target_key);
        
        while nodes.len() > 1 {
            if let Some(idx) = target_idx {
                let pair_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
                if pair_idx < nodes.len() {
                    proof.push(nodes[pair_idx].hash.clone());
                }
            }

            let mut next_level = Vec::new();
            for chunk in nodes.chunks(2) {
                let left = &chunk[0];
                let right = chunk.get(1).unwrap_or(left);

                let mut hasher = Keccak256::new();
                hasher.update(&left.hash);
                hasher.update(&right.hash);
                let parent_hash = hasher.finalize().to_vec();

                next_level.push(TreeNode {
                    key: Vec::new(),
                    value: 0,
                    hash: parent_hash,
                    left: None,
                    right: None,
                });
            }

            if let Some(idx) = target_idx {
                target_idx = Some(idx / 2);
            }
            nodes = next_level;
        }

        proof
    }

    pub fn generate_proof(&self, user_address: &[u8], token_address: &[u8]) -> (Vec<Vec<u8>>, u128, Vec<u8>) {
        let key = self.create_key(user_address, token_address);
        let value = self.leaves.get(&key).map(|node| node.value).unwrap_or(0);
        let proof = self.collect_proof(&key);
        (proof, value, key)
    }

    pub fn verify_proof(
        root: &[u8],
        proof: &[Vec<u8>],
        user_address: &[u8],
        token_address: &[u8],
        amount: u128
    ) -> bool {
        // Create leaf node hash
        let mut hasher = Keccak256::new();
        hasher.update(user_address);
        hasher.update(token_address);
        let key = hasher.finalize().to_vec();
        
        // Hash the leaf node with its value
        let mut hasher = Keccak256::new();
        hasher.update(&key);
        hasher.update(&amount.to_be_bytes());
        let mut current_hash = hasher.finalize().to_vec();

        // Traverse up the tree using the proof
        for sibling in proof {
            let mut hasher = Keccak256::new();
            // Sort the hashes to ensure consistent ordering
            if current_hash <= *sibling {
                hasher.update(&current_hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(&current_hash);
            }
            current_hash = hasher.finalize().to_vec();
        }

        // Compare with provided root
        current_hash == root
    }
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim_start_matches("0x");
    
    if hex.is_empty() {
        return Err("Empty hex string".to_string());
    }
    
    if hex.len() % 2 != 0 {
        return Err("Hex string must have an even number of characters".to_string());
    }

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid hex character found".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| e.to_string())
}

fn main(){
    let mut tree = OrderbookMerkleTree::new();
    println!("{:?}", tree.get_root());
    let user = hex_to_bytes("1234567890123456789012345678901234567890").unwrap();
    let token = hex_to_bytes("0987654321098765432109876543210987654321").unwrap();
    let token2 = hex_to_bytes("0987654321098765432109876541210987654321").unwrap();
    let token3 = hex_to_bytes("0987654321098765432109876544210987654321").unwrap();

    tree.update_balance(&user, &token, 1000);
    tree.update_balance(&user, &token2, 1000);
    tree.update_balance(&user, &token3, 1000);
    println!("{:?}", tree.get_root());
    let (proof, value, _) = tree.generate_proof(&user, &token);
    println!("Proof length: {:?}, {:?}", proof, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_address(s: &str) -> Vec<u8> {
        hex_to_bytes(s).expect("Failed to create test address")
    }

    #[test]
    fn test_proof_generation() {
        let mut tree = OrderbookMerkleTree::new();
        
        // Add multiple balances to create a non-trivial tree
        let user1 = create_test_address("1111111111111111111111111111111111111111");
        let user2 = create_test_address("2222222222222222222222222222222222222222");
        let user3 = create_test_address("3333333333333333333333333333333333333333");
        let token = create_test_address("4444444444444444444444444444444444444444");
        
        // Update balances
        tree.update_balance(&user1, &token, 1000);
        tree.update_balance(&user2, &token, 2000);
        tree.update_balance(&user3, &token, 3000);
        
        // Generate proof for user2
        let (proof, value, _) = tree.generate_proof(&user2, &token);
        
        // Check proof is not empty
        assert!(!proof.is_empty(), "Proof should not be empty");
        assert_eq!(value, 2000, "Value should match");
        
        // Verify proof length (should be log2(n) where n is number of leaves)
        assert_eq!(proof.len(), 2, "Proof length should be 2 for 3 leaves");
    }

    #[test]
    fn test_complete_flow() -> Result<(), String> {
        let mut tree = OrderbookMerkleTree::new();
        
        // Create multiple users and tokens
        let users = vec![
            hex_to_bytes("1111111111111111111111111111111111111111")?,
            hex_to_bytes("2222222222222222222222222222222222222222")?,
            hex_to_bytes("3333333333333333333333333333333333333333")?,
            hex_to_bytes("4444444444444444444444444444444444444444")?
        ];
        let token = hex_to_bytes("5555555555555555555555555555555555555555")?;

        // Update balances
        for (i, user) in users.iter().enumerate() {
            tree.update_balance(user, &token, (1000 * (i + 1)) as u128);
        }

        // Generate and verify proofs for each user
        for (i, user) in users.iter().enumerate() {
            let (proof, value, _) = tree.generate_proof(user, &token);
            assert!(!proof.is_empty(), "Proof should not be empty");
            assert_eq!(value, (1000 * (i + 1)) as u128, "Value should match");
            assert_eq!(proof.len(), 2, "Proof length should be 2 for 4 leaves");
        }

        Ok(())
    }

    #[test]
    fn test_proof_verification() {
        let mut tree = OrderbookMerkleTree::new();
        
        // Create test addresses
        let user1 = hex_to_bytes("1111111111111111111111111111111111111111").unwrap();
        let user2 = hex_to_bytes("2222222222222222222222222222222222222222").unwrap();
        let user3 = hex_to_bytes("3333333333333333333333333333333333333333").unwrap();
        let token = hex_to_bytes("4444444444444444444444444444444444444444").unwrap();
        
        // Update balances
        tree.update_balance(&user1, &token, 1000);
        tree.update_balance(&user2, &token, 2000);
        tree.update_balance(&user3, &token, 3000);
        
        // Get root
        let root = tree.get_root();
        
        // Generate proof for user2
        let (proof, value, _) = tree.generate_proof(&user2, &token);
        
        // Verify the proof
        assert!(OrderbookMerkleTree::verify_proof(
            &root,
            &proof,
            &user2,
            &token,
            value
        ), "Proof should verify successfully");
        
        // Test with wrong amount
        assert!(!OrderbookMerkleTree::verify_proof(
            &root,
            &proof,
            &user2,
            &token,
            value + 1
        ), "Proof should fail with wrong amount");
        
        // Test with wrong user
        assert!(!OrderbookMerkleTree::verify_proof(
            &root,
            &proof,
            &user1,
            &token,
            value
        ), "Proof should fail with wrong user");
        
        // Test with wrong token
        let wrong_token = hex_to_bytes("5555555555555555555555555555555555555555").unwrap();
        assert!(!OrderbookMerkleTree::verify_proof(
            &root,
            &proof,
            &user2,
            &wrong_token,
            value
        ), "Proof should fail with wrong token");
    }

    #[test]
    fn test_comprehensive_verification() {
        let mut tree = OrderbookMerkleTree::new();
        
        // Create multiple users and tokens
        let users: Vec<Vec<u8>> = (0..4).map(|i| {
            let mut addr = vec![0u8; 20];
            addr[0] = i as u8 + 1;
            addr
        }).collect();
        
        let token = vec![0u8; 20];
        
        // Update balances
        for (i, user) in users.iter().enumerate() {
            tree.update_balance(user, &token, (1000 * (i + 1)) as u128);
        }
        
        let root = tree.get_root();
        
        // Verify proofs for all users
        for (i, user) in users.iter().enumerate() {
            let (proof, value, _) = tree.generate_proof(user, &token);
            
            // Correct proof should verify
            assert!(OrderbookMerkleTree::verify_proof(
                &root,
                &proof,
                user,
                &token,
                value
            ), "Proof should verify for user {}", i);
            
            // Modified value should fail
            assert!(!OrderbookMerkleTree::verify_proof(
                &root,
                &proof,
                user,
                &token,
                value + 1
            ), "Proof should fail with modified value for user {}", i);
        }
    }
}