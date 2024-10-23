use m_proc_macros::versioned_rpc;

use crate::errors::WsResult;

#[versioned_rpc("V0_8_0", "starknet")]
pub trait StarknetWsRpcApi {
    #[subscription(name = "subscribe_foo", unsubscribe = "unsubscribe_foo", item = String, param_kind = map)]
    async fn foo(&self, block_id: starknet_core::types::BlockId) -> jsonrpsee::core::SubscriptionResult;

    #[subscription(name = "subscribeNewHeads", unsubscribe = "unsubscribe", item = starknet_api::block::BlockHeader, param_kind = map)]
    async fn subscribe_new_heads(&self, block_id: starknet_core::types::BlockId) -> WsResult;
}