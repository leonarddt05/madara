use m_proc_macros::versioned_rpc;
use mp_transactions::BroadcastedDeclareTransactionV0;

use jsonrpsee::core::RpcResult;
use starknet_core::types::DeclareTransactionResult;

/// This is an admin method, so semver is different!
#[versioned_rpc("V0_1_0", "madara")]
pub trait MadaraWriteRpcApi {
    /// Submit a new class v0 declaration transaction
    #[method(name = "addDeclareV0Transaction")]
    async fn add_declare_v0_transaction(
        &self,
        declare_transaction_v0: BroadcastedDeclareTransactionV0,
    ) -> RpcResult<DeclareTransactionResult>;
}

#[versioned_rpc("V0_1_0", "madara")]
pub trait MadaraStatusRpcApi {
    #[method(name = "ping")]
    async fn ping(&self) -> RpcResult<u64>;

    #[method(name = "shutdown")]
    async fn shutdown(&self) -> RpcResult<u64>;

    #[subscription(name = "pulse", unsubscribe = "unsubscribe", item = u64)]
    async fn pulse(&self) -> jsonrpsee::core::SubscriptionResult;
}
