use serde::Deserialize;

use zksync_env_config::{envy_load, FromEnv};

// feel free to redefine all the fields in this struct, this is just a placeholder
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BitcoinDAConfig {
    pub api_node_url: String,
    pub private_key: String,
}

impl FromEnv for BitcoinDAConfig {
    fn from_env() -> anyhow::Result<Self> {
        envy_load("bitcoinda_client", "BITCOINDA_CLIENT_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitcoinda::BitcoinDAConfig;
    use da_utils::test_utils::EnvMutex;

    static MUTEX: EnvMutex = EnvMutex::new();

    fn expected_bitcoinda_da_layer_config(pk: &str, api_node_url: &str) -> BitcoinDAConfig {
        BitcoinDAConfig {
            api_node_url: api_node_url.to_string(),
            private_key: pk.to_string(),
        }
    }

    #[test]
    fn from_env_bitcoinda_client() {
        let mut lock = MUTEX.lock();
        let config = r#"
            BITCOINDA_CLIENT_API_NODE_URL="localhost:12345"
            BITCOINDA_CLIENT_PRIVATE_KEY="0xf55baf7c0e4e33b1d78fbf52f069c426bc36cff1aceb9bc8f45d14c07f034d73"
        "#;
        unsafe { lock.set_env(config); }
        let actual = BitcoinDAConfig::from_env().unwrap();
        assert_eq!(
            actual,
            expected_bitcoinda_da_layer_config(
                "0xf55baf7c0e4e33b1d78fbf52f069c426bc36cff1aceb9bc8f45d14c07f034d73",
                "localhost:12345",
            )
        );
    }
}
