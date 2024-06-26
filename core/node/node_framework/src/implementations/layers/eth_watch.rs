use std::time::Duration;

use zksync_config::{
    configs::eth_watch::Chains::{BNB, ETH},
    ChainWatchConfig, ContractsConfig,
};
use zksync_contracts::governance_contract;
use zksync_dal::{ConnectionPool, Core};
use zksync_eth_watch::{ChainHttpQueryClient, EthWatch};
use zksync_types::{ethabi::Contract, Address};

use crate::{
    implementations::{
        layers::logger_for_testing::Log,
        resources::{
            eth_interface::EthInterfaceResource,
            pools::{MasterPool, PoolResource},
        },
    },
    service::{ServiceContext, StopReceiver},
    task::{Task, TaskId},
    wiring_layer::{WiringError, WiringLayer},
};

/// Wiring layer for ethereum watcher
///
/// Responsible for initializing and running of [`EthWatch`] component, that polls the Ethereum node for the relevant events,
/// such as priority operations (aka L1 transactions), protocol upgrades etc.
///
/// ## Requests resources
/// - [`PoolResource`] for [`MasterPool`]
/// - [`EthInterfaceResource`]
///
/// ## Adds tasks
/// - [`EthWatchTask`] (as [`Task`])
#[derive(Debug)]
pub struct ChainWatchLayer {
    // sw: changed the name from EthWatchLayer to ChainWatchLayer
    chain_watch_config: ChainWatchConfig,
    contracts_config: ContractsConfig, // sw: maybe needed to add here
}

impl ChainWatchLayer {
    pub fn new(chain_watch_config: ChainWatchConfig, contracts_config: ContractsConfig) -> Self {
        Self {
            chain_watch_config,
            contracts_config,
        }
    }
}

// sw: make changes to add bnb chain
#[async_trait::async_trait]
impl WiringLayer for ChainWatchLayer {
    fn layer_name(&self) -> &'static str {
        "chain_watch_layer" // sw: changed the name from eth_watch_layer to chain_watch layer
    }

    async fn wire(self: Box<Self>, mut context: ServiceContext<'_>) -> Result<(), WiringError> {
        // sw: this has already made the resource for running this task, now we also have to add the resource to accomodate the bnb chain
        let pool_resource = context.get_resource::<PoolResource<MasterPool>>().await?;

        // sw: dont know if these two things are clonable
        let main_pool = pool_resource.get().await.unwrap();
        // sw: i can create a new task for this here so i will be creating a new bnb task
        let client = context.get_resource::<EthInterfaceResource>().await?.0;

        let eth_client = ChainHttpQueryClient::new(
            String::from("eth_client"),
            client.clone(),
            self.contracts_config.diamond_proxy_addr,
            self.contracts_config
                .ecosystem_contracts
                .clone()
                .map(|a| a.transparent_proxy_admin_addr),
            self.contracts_config.governance_addr,
            self.chain_watch_config.confirmations_for_eth_event,
        );

        // sw: have used all the contract details of the ethereum client, need to use the contract details of the bnb chain itself.
        let bnb_client = ChainHttpQueryClient::new(
            String::from("bnb_client"),
            client.clone(),
            self.contracts_config.diamond_proxy_addr,
            self.contracts_config
                .ecosystem_contracts
                .clone()
                .map(|a| a.transparent_proxy_admin_addr),
            self.contracts_config.governance_addr,
            self.chain_watch_config.confirmations_for_bnb_event,
        );
        Log::new("eth_watch.rs", "created bnb client").log();

        context.add_task(Box::new(ChainWatchTask {
            main_pool: main_pool.clone(),
            client: eth_client,
            governance_contract: governance_contract(),
            diamond_proxy_address: self.contracts_config.diamond_proxy_addr,
            poll_interval: self.chain_watch_config.poll_interval(ETH),
        }));

        context.add_task(Box::new(ChainWatchTask {
            main_pool: main_pool.clone(),
            client: bnb_client,
            governance_contract: governance_contract(),
            diamond_proxy_address: self.contracts_config.diamond_proxy_addr,
            poll_interval: self.chain_watch_config.poll_interval(BNB),
        }));
        Log::new("eth_watch.rs", "added task to listen from the bnb chain").log();

        Ok(())
    }
}

#[derive(Debug)]
struct ChainWatchTask {
    // sw: renamed from EthWatchTask to chain watch task
    main_pool: ConnectionPool<Core>,
    client: ChainHttpQueryClient, // sw: should change the name of this aswell
    governance_contract: Contract,
    diamond_proxy_address: Address,
    poll_interval: Duration,
}

#[async_trait::async_trait]
impl Task for ChainWatchTask {
    fn id(&self) -> TaskId {
        "chain_watch".into() // sw: changed the name of the task
    }

    async fn run(self: Box<Self>, stop_receiver: StopReceiver) -> anyhow::Result<()> {
        let eth_watch = EthWatch::new(
            self.diamond_proxy_address,
            &self.governance_contract,
            Box::new(self.client),
            self.main_pool,
            self.poll_interval,
        )
        .await?;

        eth_watch.run(stop_receiver.0).await
    }
}
