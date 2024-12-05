use da_config::bitcoinda::BitcoinDAConfig;
use async_trait::async_trait;
use std::fmt::{Debug, Formatter};
use anyhow::Context;

use zksync_da_client::{types, DataAvailabilityClient};
use zksync_env_config::FromEnv;
use zksync_types::web3::keccak256;

#[derive(Clone)]
pub struct BitcoinDAClient {
    light_node_url: String,
    private_key: String,
}

impl BitcoinDAClient {
    const MAX_BLOB_SIZE: usize = 2 * 1024 * 1024; // 2 MB

    pub fn new() -> anyhow::Result<Self> {
        let config = BitcoinDAConfig::from_env()
            .context("Failed to load BitcoinDAConfig from environment")?;

        Ok(Self {
            light_node_url: config.api_node_url,
            private_key: config.private_key,
        })
    }
}

#[async_trait]
impl DataAvailabilityClient for BitcoinDAClient {
    async fn dispatch_blob(
        &self,
        _: u32, // batch_number
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, types::DAError> {
        if data.len() > BitcoinDAClient::MAX_BLOB_SIZE {
            return Err(types::DAError::InvalidData(
                "Blob size exceeds the maximum limit".to_string(),
            ));
        }

        // Hash the blob
        let hash = keccak256(&data);

        // TODO: Send data to Syscoin client and await confirmation
        Ok(types::DispatchResponse { blob_id: hash })
    }

    async fn get_inclusion_data(
        &self,
        _blob_id: &str,
    ) -> anyhow::Result<Option<types::InclusionData>, types::DAError> {
        // Return the hash of the pubdata
        Ok(Some(types::InclusionData {
            data: _blob_id.as_bytes().to_vec(),
        }))
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> Option<usize> {
        Some(BitcoinDAClient::MAX_BLOB_SIZE)
    }
}

impl Debug for BitcoinDAClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitcoinDAClient")
            .field("light_node_url", &self.light_node_url)
            .finish()
    }
}
