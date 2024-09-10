use std::collections::HashMap;

use alloy_primitives::{keccak256, Bytes, B256};
use anyhow::{anyhow, bail};
use ethportal_api::{
    types::{
        content_key::state::{AccountTrieNodeKey, ContractBytecodeKey, ContractStorageTrieNodeKey},
        content_value::state::{
            AccountTrieNodeWithProof, ContractBytecodeWithProof, ContractStorageTrieNodeWithProof,
        },
        execution::header_with_proof::{
            BlockHeaderProof, HeaderWithProof, PreMergeAccumulatorProof,
        },
        state_trie::{
            nibbles::Nibbles,
            trie_traversal::{NodeTraversal, TraversalResult},
            ByteCode, EncodedTrieNode, TrieProof,
        },
    },
    BlockHeaderKey, HistoryContentKey, HistoryContentValue, StateContentKey, StateContentValue,
};

use crate::types::BlockData;

pub fn create_history_content(
    block_data: &BlockData,
) -> HashMap<HistoryContentKey, HistoryContentValue> {
    let header_key =
        HistoryContentKey::BlockHeaderWithProof(BlockHeaderKey::from(block_data.header.hash()));
    let header_value = HistoryContentValue::BlockHeaderWithProof(HeaderWithProof {
        header: block_data.header.clone(),
        proof: BlockHeaderProof::PreMergeAccumulatorProof(PreMergeAccumulatorProof {
            proof: block_data.header_accumulator_proof,
        }),
    });

    HashMap::from([(header_key, header_value)])
}

pub fn create_state_content(
    block_data: &BlockData,
) -> anyhow::Result<HashMap<StateContentKey, StateContentValue>> {
    let block_hash = block_data.header.hash();

    let mut result = HashMap::new();
    for account_state in &block_data.state {
        let address_hash = keccak256(account_state.address);

        // account trie
        create_state_content_from_trie_proof(
            address_hash,
            &account_state.account_proof,
            |node_hash, path| {
                StateContentKey::AccountTrieNode(AccountTrieNodeKey { path, node_hash })
            },
            |trie_proof| {
                StateContentValue::AccountTrieNodeWithProof(AccountTrieNodeWithProof {
                    proof: trie_proof,
                    block_hash,
                })
            },
            &mut result,
        )?;

        let account_proof = account_state
            .account_proof
            .iter()
            .map(|trie_node| EncodedTrieNode::from(trie_node.to_vec()))
            .collect::<Vec<_>>();
        let account_proof =
            TrieProof::new(account_proof.clone()).map_err(|err| anyhow!("{err:?}"))?;

        // storage code
        if let Some(code) = &account_state.code {
            assert_eq!(
                account_state.code_hash,
                keccak256(code),
                "CodeHash doesn't match"
            );
            result.insert(
                StateContentKey::ContractBytecode(ContractBytecodeKey {
                    address_hash,
                    code_hash: account_state.code_hash,
                }),
                StateContentValue::ContractBytecodeWithProof(ContractBytecodeWithProof {
                    code: ByteCode::new(code.to_vec()).map_err(|err| anyhow!("{err:?}"))?,
                    account_proof: account_proof.clone(),
                    block_hash,
                }),
            );
        }

        // storage trie
        for storage_item in &account_state.storage_proof {
            let key = keccak256(storage_item.key.to_be_bytes::<32>());
            create_state_content_from_trie_proof(
                key,
                &storage_item.proof,
                |node_hash, path| {
                    StateContentKey::ContractStorageTrieNode(ContractStorageTrieNodeKey {
                        address_hash,
                        path,
                        node_hash,
                    })
                },
                |tire_proof| {
                    StateContentValue::ContractStorageTrieNodeWithProof(
                        ContractStorageTrieNodeWithProof {
                            storage_proof: tire_proof,
                            account_proof: account_proof.clone(),
                            block_hash,
                        },
                    )
                },
                &mut result,
            )?;
        }
    }
    Ok(result)
}

fn create_state_content_from_trie_proof(
    key: B256,
    proof: &[Bytes],
    content_key_fn: impl Fn(B256, Nibbles) -> StateContentKey,
    content_value_fn: impl Fn(TrieProof) -> StateContentValue,
    contents: &mut HashMap<StateContentKey, StateContentValue>,
) -> anyhow::Result<()> {
    let nibbles = Nibbles::unpack_nibbles(key.as_slice());
    let proof = proof
        .iter()
        .map(|trie_node| EncodedTrieNode::from(trie_node.to_vec()))
        .collect::<Vec<_>>();

    let mut remaining_path = nibbles.as_slice();
    for (i, trie_node) in proof.iter().enumerate() {
        let Some(path) = nibbles.strip_suffix(remaining_path) else {
            bail!("remaining_path should be suffix of nibbles");
        };

        contents
            .entry(content_key_fn(
                trie_node.node_hash(),
                Nibbles::try_from_unpacked_nibbles(path)?,
            ))
            .or_insert_with(|| content_value_fn(TrieProof::from(proof[..=i].to_vec())));

        // update remaining_path as preparation for the next node
        remaining_path = match trie_node.as_trie_node()?.traverse(remaining_path) {
            TraversalResult::Node(next_node) => next_node.remaining_path,
            TraversalResult::Value(_) => &[],
            traversal_result => {
                bail!("Unexpected trie traversal result: {traversal_result:?}")
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(483333)]
    fn history(#[case] block_number: u64) {
        let block_data = BlockData::from_file(block_number).unwrap();
        assert!(!create_history_content(&block_data).is_empty());
    }

    #[rstest]
    #[case(483333)]
    fn state(#[case] block_number: u64) {
        let block_data = BlockData::from_file(block_number).unwrap();
        assert!(create_state_content(&block_data).is_ok());
    }
}
