use std::{fs::File, io::BufReader};

use alloy_primitives::{Address, Bytes, B256, U256};
use ethportal_api::Header;
use serde::{de::Error, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockData {
    pub header: Header,
    pub header_accumulator_proof: [B256; 15],
    pub state: Vec<AccountState>,
}

impl BlockData {
    pub fn from_file(block: u64) -> Result<Self, serde_json::Error> {
        let file = File::open(format!("data/{block}.json"))
            .map_err(|err| serde_json::Error::custom(format!("Error opening file: {err}")))?;
        serde_json::from_reader(BufReader::new(file))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountState {
    pub address: Address,
    pub account_proof: Vec<Bytes>,
    pub balance: U256,
    pub code_hash: B256,
    pub code: Option<Bytes>,
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
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(483333)]
    fn parse_file(#[case] block_number: u64) {
        let block_data = BlockData::from_file(block_number).unwrap();
        assert_eq!(block_data.header.number, block_number);
    }
}
