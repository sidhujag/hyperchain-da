use da_config::bitcoinda::BitcoinDAConfig;
use async_trait::async_trait;
use std::fmt::{Debug, Formatter};

use zksync_da_client::{types, DataAvailabilityClient};
use zksync_env_config::FromEnv;

use zksync_types::web3::keccak256;

#[derive(Clone)]
pub struct BitcoinDAClient {
    light_node_url: String,
    private_key: String,
}

impl BitcoinDAClient {
    const MAX_BLOB_SIZE: usize = 2 * 1024 * 1024; // 2 mb
    pub fn new() -> anyhow::Result<Self> {
        // TODO: read proto config first
        let config = BitcoinDAConfig::from_env()?;

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
        // hash the blob
        let hash = keccak256(&data);
        // send data to syscoin client, await its confirmation via txid
        Ok(DispatchResponse {
            blob_id: hash
        })
    }
    async fn get_inclusion_data(
        &self,
        _blob_id: &str,
    ) -> anyhow::Result<Option<types::InclusionData>, types::DAError> {
        // return the hash of the pubdata which accompanied with the uncompressed state diff hash should enable us to recreate output hash from validatePubData
        Ok(Some(InclusionData { data: _blob_id.as_bytes().to_vec() }))
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) ->  Option<usize> {
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
