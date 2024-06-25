use std::time::Duration;

use serde::{Deserialize, Serialize};

pub enum Chains {
    ETH,
    BNB,
}

/// Configuration for the Ethereum watch crate.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChainWatchConfig {
    /// Amount of confirmations for the priority operation to be processed.
    /// If not specified operation will be processed once its block is finalized.
    pub confirmations_for_eth_event: Option<u64>,
    /// How often we want to poll the Ethereum node.
    /// Value in milliseconds.
    pub eth_node_poll_interval: u64,
    /// Amount of confirmations for the priority operation to be processed.
    /// If not specified operation will be processed once its block is finalized.
    pub confirmations_for_bnb_event: Option<u64>,
    /// How often we want to poll the Ethereum node.
    /// Value in milliseconds.
    pub bnb_node_poll_interval: u64,
}

use Chains::{BNB, ETH};
impl ChainWatchConfig {
    /// Converts `self.eth_node_poll_interval` into `Duration`.
    pub fn poll_interval(&self, chain: Chains) -> Duration {
        match chain {
            ETH => Duration::from_millis(self.eth_node_poll_interval),
            BNB => Duration::from_millis(self.bnb_node_poll_interval),
        }
    }
}

#[test]
fn poll_interval() {
    let chain_watch_config = ChainWatchConfig {
        confirmations_for_bnb_event: Some(1),
        confirmations_for_eth_event: Some(1),
        bnb_node_poll_interval: 3,
        eth_node_poll_interval: 12,
    };
    let duration = chain_watch_config.poll_interval(ETH);
    assert_eq!(duration, Duration::from_millis(12));
    let duration = chain_watch_config.poll_interval(BNB);
    assert_eq!(duration, Duration::from_millis(3));
}
