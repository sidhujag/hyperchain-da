use da_config::bitcoinda::BitcoinDAConfig;
use async_trait::async_trait;
use std::fmt::{Debug, Formatter};

use zksync_env_config::FromEnv;


use std::{
    io::{Read, Write},
    sync::Arc,
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use zksync_config::ObjectStoreConfig;
use zksync_da_client::{
    types::{DAError, DispatchResponse, InclusionData},
    DataAvailabilityClient,
};
use zksync_object_store::{
    Bucket, ObjectStore, ObjectStoreFactory, StoredObject, _reexports::BoxedError,
};

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


/// An implementation of the `DataAvailabilityClient` trait that stores the pubdata in the GCS.
#[derive(Clone, Debug)]
pub struct ObjectStoreDAClient {
    object_store: Arc<dyn ObjectStore>,
}

impl ObjectStoreDAClient {
    pub async fn new(object_store_conf: ObjectStoreConfig) -> anyhow::Result<Self> {
        Ok(ObjectStoreDAClient {
            object_store: ObjectStoreFactory::new(object_store_conf)
                .create_store()
                .await?,
        })
    }
}

#[async_trait]
impl DataAvailabilityClient for BitcoinDAClient {
    async fn dispatch_blob(
        &self,
        _: u32, // batch_number
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError> {
        // hash the blob
        let hash = keccak256(data);
        // send blob to GCS
        if let Err(err) = self
            .object_store
            .put(hash, &StorablePubdata { data })
            .await
        {
            return Err(DAError {
                is_retriable: err.is_retriable(),
                error: anyhow::Error::from(err),
            });
        }

        // send data to syscoin client, await its confirmation via txid
        Ok(DispatchResponse {
            blob_id: hash
        })
    }
    async fn get_inclusion_data(
        &self,
        _blob_id: &str,
    ) -> anyhow::Result<Option<InclusionData>, DAError> {
        if let Err(err) = self
        .object_store
        .get::<StorablePubdata>(_blob_id)
        .await
    {
        if let zksync_object_store::ObjectStoreError::KeyNotFound(_) = err {
            return Ok(None);
        }

        return Err(DAError {
            is_retriable: err.is_retriable(),
            error: anyhow::Error::from(err),
        });
    }

    // Using default here because we don't get any inclusion data from object store, thus
    // there's nothing to check on L1.
    return Ok(Some(InclusionData::default()));
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



/// Used as a wrapper for the pubdata to be stored in the GCS.
#[derive(Debug)]
struct StorablePubdata {
    pub data: Vec<u8>,
}

impl StoredObject for StorablePubdata {
    const BUCKET: Bucket = Bucket::DataAvailability;
    type Key<'a> = String;

    fn encode_key(key: Self::Key<'_>) -> String {
        format!("l1_batch_{key}_pubdata.gzip")
    }

    fn serialize(&self) -> Result<Vec<u8>, BoxedError> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.data[..])?;
        encoder.finish().map_err(From::from)
    }

    fn deserialize(bytes: Vec<u8>) -> Result<Self, BoxedError> {
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut decompressed_bytes = Vec::new();
        decoder
            .read_to_end(&mut decompressed_bytes)
            .map_err(BoxedError::from)?;

        Ok(Self {
            data: decompressed_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use tokio::fs;
    use zksync_object_store::{MockObjectStore, StoredObject};

    use super::StorablePubdata;

    #[tokio::test]
    async fn test_storable_pubdata_deserialization() {
        let serialized = fs::read("./src/test_data/l1_batch_123_pubdata.gzip")
            .await
            .unwrap();

        let data = StorablePubdata::deserialize(serialized).unwrap().data;
        assert_eq!(data[12], 0);
        assert_eq!(data[123], 129);
        assert_eq!(data[1234], 153);
    }

    #[tokio::test]
    async fn stored_object_serialization() {
        let data = vec![1, 2, 3, 4, 5, 6, 123, 255, 0, 0];
        let hash = keccak256(data);

        let store = MockObjectStore::arc();
        store
            .put(
                hash,
                &StorablePubdata { data: data.clone() },
            )
            .await
            .unwrap();

        let resp = store
            .get::<StorablePubdata>(hash)
            .await
            .unwrap();

        assert_eq!(data, resp.data);
    }
}
