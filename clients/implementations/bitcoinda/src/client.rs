use da_config::bitcoinda::BitcoinDAConfig;
use async_trait::async_trait;
use std::fmt::{Debug, Formatter};

use zksync_da_client::{types, DataAvailabilityClient};
use zksync_env_config::FromEnv;

#[derive(Clone)]
pub struct BitcoinDAClient {
    light_node_url: String,
    private_key: String,
}

impl BitcoinDAClient {
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
        _: Vec<u8>, // data
    ) -> Result<types::DispatchResponse, types::DAError> {
        todo!()
    }
    async fn get_inclusion_data(
        &self,
        _blob_id: &str,
    ) -> anyhow::Result<Option<types::InclusionData>, types::DAError> {
        todo!()
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) ->  Option<usize> {
        Some(1973786)
    }
}

impl Debug for BitcoinDAClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitcoinDAClient")
            .field("light_node_url", &self.light_node_url)
            .finish()
    }
}