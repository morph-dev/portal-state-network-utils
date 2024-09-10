use std::time::Duration;

use alloy_primitives::Bytes;
use anyhow::Result;
use clap::Parser;
use ethportal_api::{jsonrpsee::http_client::HttpClientBuilder, ContentValue, OverlayContentKey};
use portal_state_network::{
    portal::content_creation::{create_history_content, create_state_content},
    types::BlockData,
};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub block: u64,
    #[arg(long)]
    pub trin_client: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let block_data = BlockData::from_file(args.block).unwrap();

    let client = HttpClientBuilder::default()
        .request_timeout(Duration::from_secs(5))
        .build(args.trin_client)?;

    for (key, value) in create_history_content(&block_data) {
        use ethportal_api::HistoryNetworkApiClient;
        print!(
            "Gossiping history content: {} ...",
            Bytes::from(key.to_bytes())
        );
        let result = client.gossip(key, value.encode()).await;
        println!(" Result: {result:?}");
    }

    for (key, value) in create_state_content(&block_data)? {
        use ethportal_api::StateNetworkApiClient;
        print!(
            "Gossiping state content: {} ...",
            Bytes::from(key.to_bytes())
        );
        let result = client.gossip(key, value.encode()).await;
        println!(" Result: {result:?}");
    }

    Ok(())
}
