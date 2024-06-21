use zksync_config::ChainWatchConfig;

use crate::{envy_load, FromEnv};

impl FromEnv for ChainWatchConfig {
    fn from_env() -> anyhow::Result<Self> {
        envy_load("eth_watch", "CHAIN_WATCH_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::EnvMutex;

    static MUTEX: EnvMutex = EnvMutex::new();

    fn expected_config() -> ChainWatchConfig {
        ChainWatchConfig {
            confirmations_for_eth_event: Some(0),
            eth_node_poll_interval: 300,
            bnb_node_poll_interval: 300,
            confirmations_for_bnb_event: Some(0)
        }
    }

    #[test]
    fn from_env() {
        let mut lock = MUTEX.lock();
        let config = r#"
            CHAIN_WATCH_CONFIRMATIONS_FOR_ETH_EVENT="0"
            CHAIN_WATCH_ETH_NODE_POLL_INTERVAL="300"
            CHAIN_WATCH_BNB_NODE_POLL_INTERVAL="300"
            CHAIN_WATCH_CONFIRMATIONS_FOR_BNB_EVENT="0"
        "#;
        lock.set_env(config);

        let actual = ChainWatchConfig::from_env().unwrap();
        assert_eq!(actual, expected_config());
    }
}
