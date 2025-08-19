use anyhow::Result;
use serde::{Deserialize, Serialize};
use starknet::core::crypto::compute_hash_on_elements;
use starknet::core::types::Felt;
use std::collections::HashMap;

/// Type alias for merkle tree result: (root, proofs)
pub type MerkleTreeResult = (Vec<u8>, HashMap<String, Vec<String>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleClaimData {
    pub address: String,
    pub token_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub address: String,
    pub proof: Vec<String>,
}

/// Compute the leaf hash for a merkle tree entry using Poseidon hash
/// Following the same pattern as the JS implementation
fn compute_leaf_hash(address: &str, token_ids: &[i64]) -> Result<Felt> {
    // Parse address to Felt
    let address_felt = Felt::from_hex(address)
        .or_else(|_| Felt::from_dec_str(address))
        .map_err(|e| anyhow::anyhow!("Failed to parse address {}: {}", address, e))?;

    // Build elements array: [address, token_ids.length, ...token_ids, 0]
    // The trailing 0 is for backwards compatibility with og_token_ids
    let mut elements = vec![address_felt];

    // Add token_ids length and values
    elements.push(Felt::from(token_ids.len() as u64));
    for id in token_ids {
        elements.push(Felt::from(*id as u64));
    }

    // Add empty og_token_ids for compatibility (length = 0)
    elements.push(Felt::from(0u64));

    // Compute Poseidon hash on elements
    Ok(compute_hash_on_elements(&elements))
}

/// Build a merkle tree from claim data and return the root and proofs
pub fn build_merkle_tree(claims: &[MerkleClaimData]) -> Result<MerkleTreeResult> {
    if claims.is_empty() {
        return Err(anyhow::anyhow!("Cannot build merkle tree with no claims"));
    }

    // Compute leaf hashes
    let mut leaf_hashes: Vec<(String, Felt)> = Vec::new();
    for claim in claims {
        let hash = compute_leaf_hash(&claim.address, &claim.token_ids)?;
        leaf_hashes.push((claim.address.clone(), hash));
    }

    // Sort leaf hashes for consistent tree construction
    leaf_hashes.sort_by(|a, b| a.1.cmp(&b.1));

    // Build the merkle tree
    let mut proofs: HashMap<String, Vec<String>> = HashMap::new();

    // If there's only one leaf, the root is the leaf itself
    if leaf_hashes.len() == 1 {
        let root = leaf_hashes[0].1;
        proofs.insert(leaf_hashes[0].0.clone(), vec![]);
        return Ok((root.to_bytes_be().to_vec(), proofs));
    }

    // Build tree levels
    let mut current_level: Vec<Felt> = leaf_hashes.iter().map(|(_, h)| *h).collect();
    let mut tree_levels: Vec<Vec<Felt>> = vec![current_level.clone()];

    // Build tree from bottom to top
    while current_level.len() > 1 {
        let mut next_level = Vec::new();

        for i in (0..current_level.len()).step_by(2) {
            if i + 1 < current_level.len() {
                // Sort the pair before hashing (matching strk-merkle-tree implementation)
                let mut pair = [current_level[i], current_level[i + 1]];
                pair.sort();
                let hash = compute_hash_on_elements(&pair);
                next_level.push(hash);
            } else {
                // Odd number of nodes, hash with 0x0 (sorted if necessary)
                let mut pair = [current_level[i], Felt::ZERO];
                pair.sort();
                let hash = compute_hash_on_elements(&pair);
                next_level.push(hash);
            }
        }

        tree_levels.push(next_level.clone());
        current_level = next_level;
    }

    let root = current_level[0];

    // Generate proofs for each leaf
    for (idx, (address, _leaf_hash)) in leaf_hashes.iter().enumerate() {
        let mut proof = Vec::new();
        let mut current_idx = idx;

        // Traverse tree levels from bottom to top (excluding root level)
        for level in &tree_levels[..tree_levels.len() - 1] {
            // Find sibling
            if current_idx % 2 == 0 {
                // Even index, sibling is on the right (or 0x0 if no sibling)
                if current_idx + 1 < level.len() {
                    proof.push(format!("0x{:064x}", level[current_idx + 1]));
                } else {
                    // No sibling, use 0x0 (matching starknet.js implementation)
                    proof.push(format!("0x{:064x}", Felt::ZERO));
                }
            } else {
                // Odd index, sibling is on the left
                proof.push(format!("0x{:064x}", level[current_idx - 1]));
            }
            current_idx /= 2;
        }

        proofs.insert(address.clone(), proof);
    }

    Ok((root.to_bytes_be().to_vec(), proofs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_hash() {
        let claim = MerkleClaimData {
            address: "0x123".to_string(),
            token_ids: vec![1, 2, 3],
        };

        let hash = compute_leaf_hash(&claim.address, &claim.token_ids);
        assert!(hash.is_ok());
    }

    #[test]
    fn test_merkle_tree_single_claim() {
        let claims = vec![MerkleClaimData {
            address: "0x123".to_string(),
            token_ids: vec![1],
        }];

        let result = build_merkle_tree(&claims);
        assert!(result.is_ok());

        let (root, proofs) = result.unwrap();
        assert!(!root.is_empty());
        assert_eq!(proofs.len(), 1);
        assert!(proofs.contains_key("0x123"));
    }

    #[test]
    fn test_merkle_tree_multiple_claims() {
        let claims = vec![
            MerkleClaimData {
                address: "0x123".to_string(),
                token_ids: vec![1, 2],
            },
            MerkleClaimData {
                address: "0x456".to_string(),
                token_ids: vec![3, 4],
            },
        ];

        let result = build_merkle_tree(&claims);
        assert!(result.is_ok());

        let (root, proofs) = result.unwrap();
        assert!(!root.is_empty());
        assert_eq!(proofs.len(), 2);
        assert!(proofs.contains_key("0x123"));
        assert!(proofs.contains_key("0x456"));
    }
}
