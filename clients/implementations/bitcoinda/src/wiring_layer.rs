use crate::{client::BitcoinDAClient};


use zksync_da_client::DataAvailabilityClient;
use zksync_node_framework::implementations::resources::da_client::DAClientResource;
use zksync_node_framework::{IntoContext, wiring_layer::{WiringError, WiringLayer}};

#[derive(Debug, Default)]
pub struct BitcoinDAWiringLayer;

#[derive(Debug, IntoContext)]
pub struct Output {
    pub client: DAClientResource,
}


#[async_trait::async_trait]
impl WiringLayer for BitcoinDAWiringLayer {
    type Input = ();
    type Output = Output;

    fn layer_name(&self) -> &'static str {
        "bitcoinda_client_layer"
    }

    async fn wire(self, _: Self::Input) -> Result<Self::Output, WiringError> {
        let client: Box<dyn DataAvailabilityClient> = Box::new(BitcoinDAClient::new()?);

        Ok(Self::Output {
            client: DAClientResource(client),
        })
    }
}
