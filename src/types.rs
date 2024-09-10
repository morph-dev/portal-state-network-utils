use alloy_primitives::{Address, Bytes, B256, U256};
use ethportal_api::Header;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockData {
    pub header: Header,
    pub state: Vec<AccountState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountState {
    pub address: Address,
    pub account_proof: Vec<Bytes>,
    pub balance: U256,
    pub code_hash: B256,
    #[serde(with = "alloy_serde::quantity")]
    pub nonce: u64,
    pub storage_hash: B256,
    pub storage_proof: Vec<StorageItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageItem {
    pub key: U256,
    pub value: U256,
    pub proof: Vec<Bytes>,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader, path::PathBuf};

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(483333)]
    fn parse_file(#[case] block_number: u64) {
        let filepath =
            File::open(PathBuf::from("data").join(format!("{block_number}.json"))).unwrap();
        let block_data: BlockData = serde_json::from_reader(BufReader::new(filepath)).unwrap();
        assert_eq!(block_data.header.number, block_number);
    }
}
