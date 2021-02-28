//! A collection of node-specific RPC methods.

use std::{sync::Arc};

use fc_rpc_core::types::{PendingTransactions, FilterPool};
use sgc_runtime::{Hash, AccountId, Index, opaque::Block, BlockNumber, Balance};
use sp_api::ProvideRuntimeApi;
use sp_transaction_pool::TransactionPool;
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use sc_rpc_api::DenyUnsafe;
use sc_client_api::{
    backend::{StorageProvider, Backend, StateBackend, AuxStore},
    client::BlockchainEvents
};
use sc_rpc::SubscriptionTaskExecutor;
use sp_runtime::traits::BlakeTwo256;
use sp_block_builder::BlockBuilder;
use sc_network::NetworkService;
use jsonrpc_pubsub::manager::SubscriptionManager;

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// The Node authority flag
    pub is_authority: bool,
    /// Whether to enable dev signer
    pub enable_dev_signer: bool,
    /// Network service
    pub network: Arc<NetworkService<Block, Hash>>,
    /// Ethereum pending transactions.
    pub pending_transactions: PendingTransactions,
    /// EthFilterApi pool.
    pub filter_pool: Option<FilterPool>,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, BE>(
    deps: FullDeps<C, P>,
    subscription_task_executor: SubscriptionTaskExecutor
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata> where
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: BlockchainEvents<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error=BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: BlockBuilder<Block>,
    C::Api: pallet_contracts_rpc::ContractsRuntimeApi<Block, AccountId, Balance, BlockNumber>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    P: TransactionPool<Block=Block> + 'static,
{
    use substrate_frame_rpc_system::{FullSystem, SystemApi};
    use pallet_contracts_rpc::{Contracts, ContractsApi};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use fc_rpc::{
        EthApi, EthApiServer, EthFilterApi, EthFilterApiServer, NetApi, NetApiServer,
        EthPubSubApi, EthPubSubApiServer, Web3Api, Web3ApiServer, EthDevSigner, EthSigner,
        HexEncodedIdProvider,
    };

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        deny_unsafe,
        is_authority,
        network,
        pending_transactions,
        filter_pool,
        enable_dev_signer,
    } = deps;

    io.extend_with(
        SystemApi::to_delegate(FullSystem::new(client.clone(), pool.clone(), deny_unsafe))
    );
    io.extend_with(
        ContractsApi::to_delegate(Contracts::new(client.clone()))
    );
    io.extend_with(
        TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone()))
    );

    let mut signers = Vec::new();
    if enable_dev_signer {
        signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
    }
    io.extend_with(
        EthApiServer::to_delegate(EthApi::new(
            client.clone(),
            pool.clone(),
            sgc_runtime::TransactionConverter,
            network.clone(),
            pending_transactions.clone(),
            signers,
            is_authority,
        ))
    );

    if let Some(filter_pool) = filter_pool {
        io.extend_with(
            EthFilterApiServer::to_delegate(EthFilterApi::new(
                client.clone(),
                filter_pool.clone(),
                500 as usize, // max stored filters
            ))
        );
    }

    io.extend_with(
        NetApiServer::to_delegate(NetApi::new(
            client.clone(),
            network.clone(),
        ))
    );

    io.extend_with(
        Web3ApiServer::to_delegate(Web3Api::new(
            client.clone(),
        ))
    );

    io.extend_with(
        EthPubSubApiServer::to_delegate(EthPubSubApi::new(
            pool.clone(),
            client.clone(),
            network.clone(),
            SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
                HexEncodedIdProvider::default(),
                Arc::new(subscription_task_executor)
            ),
        ))
    );

    io
}
