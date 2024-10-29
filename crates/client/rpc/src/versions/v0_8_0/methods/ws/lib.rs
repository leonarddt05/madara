use crate::{
    errors::{ErrorExtWs, OptionExtWs, StarknetWsApiError},
    versions::v0_8_0::StarknetWsRpcApiV0_8_0Server,
};

use super::BLOCK_PAST_LIMIT;

#[jsonrpsee::core::async_trait]
impl StarknetWsRpcApiV0_8_0Server for crate::Starknet {
    async fn subscribe_new_heads(
        &self,
        pending: jsonrpsee::PendingSubscriptionSink,
        block_id: starknet_core::types::BlockId,
    ) -> jsonrpsee::core::SubscriptionResult {
        let sink = pending.accept().await.or_internal_server_error("Failed to establish websocket connection")?;

        let mut block_n = match block_id {
            starknet_core::types::BlockId::Number(block_n) => {
                let err = || format!("Failed to retrieve block info for block {block_n}");
                let block_latest = self
                    .backend
                    .get_block_n(&mp_block::BlockId::Tag(mp_block::BlockTag::Latest))
                    .or_else_internal_server_error(err)?
                    .ok_or(StarknetWsApiError::NoBlocks)?;

                if block_n < block_latest.saturating_sub(BLOCK_PAST_LIMIT) {
                    return Err(StarknetWsApiError::TooManyBlocksBack.into());
                }

                block_n
            }
            starknet_core::types::BlockId::Hash(block_hash) => {
                let err = || format!("Failed to retrieve block info at hash {block_hash:#x}");
                let block_latest = self
                    .backend
                    .get_block_n(&mp_block::BlockId::Tag(mp_block::BlockTag::Latest))
                    .or_else_internal_server_error(err)?
                    .ok_or(StarknetWsApiError::BlockNotFound)?;

                let block_n = self
                    .backend
                    .get_block_n(&block_id)
                    .or_else_internal_server_error(err)?
                    .ok_or_else_internal_server_error(err)?;

                if block_n < block_latest.saturating_sub(BLOCK_PAST_LIMIT) {
                    return Err(StarknetWsApiError::TooManyBlocksBack.into());
                }

                block_n
            }
            starknet_core::types::BlockId::Tag(starknet_core::types::BlockTag::Latest) => self
                .backend
                .get_latest_block_n()
                .or_internal_server_error("Failed to retrieve block info for latest block")?
                .ok_or_internal_server_error("Failed to retrieve block info for latest block")?,
            starknet_core::types::BlockId::Tag(starknet_core::types::BlockTag::Pending) => {
                return Err(StarknetWsApiError::Pending.into());
            }
        };

        let mut rx = self.backend.subscribe_block_info();
        for n in block_n.. {
            if sink.is_closed() {
                return Ok(());
            }

            let block_info = match self.backend.get_block_info(&mp_block::BlockId::Number(n)) {
                Ok(Some(block_info)) => {
                    let err = format!("Failed to retrieve block info for block {n}");
                    block_info.as_nonpending_owned().ok_or_internal_server_error(err)?
                }
                Ok(None) => break,
                Err(e) => {
                    let err = format!("Failed to retrieve block info for block {n}: {e}");
                    return Err(StarknetWsApiError::internal_server_error(err).into());
                }
            };

            send_block_header(&sink, block_info, block_n).await?;
            block_n = block_n.saturating_add(1);
        }

        // We need to check the block number at each iteration as the first
        // time this is exectued we might already have received some blocks
        // from the backend which we manually fecthed from db
        loop {
            tokio::select! {
                block_info = rx.recv() => {
                    let block_info = block_info.or_internal_server_error("Failed to retrieve block info")?;
                    if block_info.header.block_number == block_n {
                        send_block_header(&sink, block_info, block_n).await?;
                    }
                    block_n = block_n.saturating_add(1);
                },
                _ = sink.closed() => {
                    return Ok(())
                }
            }
        }
    }
}

async fn send_block_header<'a>(
    sink: &jsonrpsee::core::server::SubscriptionSink,
    block_info: mp_block::MadaraBlockInfo,
    block_n: u64,
) -> Result<(), StarknetWsApiError> {
    let header = starknet_types_rpc::BlockHeader::from(block_info);
    let msg = jsonrpsee::SubscriptionMessage::from_json(&header)
        .or_else_internal_server_error(|| format!("Failed to create response message for block {block_n}"))?;

    sink.send(msg).await.or_internal_server_error("Failed to respond to websocket request")?;

    Ok(())
}

#[cfg(test)]
mod test {
    use jsonrpsee::ws_client::WsClientBuilder;
    use starknet_core::types::Felt;

    use crate::{
        test_utils::rpc_test_setup,
        versions::v0_8_0::{
            methods::ws::BLOCK_PAST_LIMIT, NewHead, StarknetWsRpcApiV0_8_0Client, StarknetWsRpcApiV0_8_0Server,
        },
        Starknet,
    };

    fn block_generator(backend: &mc_db::MadaraBackend) -> impl Iterator<Item = NewHead> + '_ {
        (0..).map(|n| {
            backend
                .store_block(
                    mp_block::MadaraMaybePendingBlock {
                        info: mp_block::MadaraMaybePendingBlockInfo::NotPending(mp_block::MadaraBlockInfo {
                            header: mp_block::Header {
                                parent_block_hash: Felt::from(n),
                                block_number: n,
                                ..Default::default()
                            },
                            block_hash: Felt::from(n),
                            tx_hashes: vec![],
                        }),
                        inner: mp_block::MadaraBlockInner { transactions: vec![], receipts: vec![] },
                    },
                    mp_state_update::StateDiff::default(),
                    vec![],
                )
                .expect("Storing block");

            let block_info = backend
                .get_block_info(&mp_block::BlockId::Number(n))
                .expect("Retrieving block info")
                .expect("Retrieving block info")
                .as_nonpending_owned()
                .expect("Retrieving block info");

            NewHead::from(block_info)
        })
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn subscribe_new_heads(rpc_test_setup: (std::sync::Arc<mc_db::MadaraBackend>, Starknet)) {
        let (backend, starknet) = rpc_test_setup;
        let server = jsonrpsee::server::Server::builder().build("127.0.0.1:0").await.expect("Starting server");
        let server_url = format!("ws://{}", server.local_addr().expect("Retrieving server local address"));
        // Server will be stopped once this is dropped
        let _server_handle = server.start(StarknetWsRpcApiV0_8_0Server::into_rpc(starknet));
        let client = WsClientBuilder::default().build(&server_url).await.expect("Building client");

        let mut generator = block_generator(&backend);
        let expected = generator.next().expect("Retrieving block from backend");

        let mut sub = client
            .subscribe_new_heads(starknet_core::types::BlockId::Tag(starknet_core::types::BlockTag::Latest))
            .await
            .expect("starknet_subscribeNewHeads");

        let next = sub.next().await;
        let header = next.expect("Waiting for block header").expect("Waiting for block header");

        assert_eq!(
            header,
            expected,
            "actual: {}\nexpect: {}",
            serde_json::to_string_pretty(&header).unwrap_or_default(),
            serde_json::to_string_pretty(&expected).unwrap_or_default()
        );
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn subscribe_new_heads_many(rpc_test_setup: (std::sync::Arc<mc_db::MadaraBackend>, Starknet)) {
        let (backend, starknet) = rpc_test_setup;
        let server = jsonrpsee::server::Server::builder().build("127.0.0.1:0").await.expect("Starting server");
        let server_url = format!("ws://{}", server.local_addr().expect("Retrieving server local address"));
        // Server will be stopped once this is dropped
        let _server_handle = server.start(StarknetWsRpcApiV0_8_0Server::into_rpc(starknet));
        let client = WsClientBuilder::default().build(&server_url).await.expect("Building client");

        let generator = block_generator(&backend);
        let expected: Vec<_> = generator.take(BLOCK_PAST_LIMIT as usize).collect();

        let mut sub = client
            .subscribe_new_heads(starknet_core::types::BlockId::Number(0))
            .await
            .expect("starknet_subscribeNewHeads");

        for e in expected {
            let next = sub.next().await;
            let header = next.expect("Waiting for block header").expect("Waiting for block header");

            assert_eq!(
                header,
                e,
                "actual: {}\nexpect: {}",
                serde_json::to_string_pretty(&header).unwrap_or_default(),
                serde_json::to_string_pretty(&e).unwrap_or_default()
            );
        }
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn subscribe_new_heads_disconnect(rpc_test_setup: (std::sync::Arc<mc_db::MadaraBackend>, Starknet)) {
        let (backend, starknet) = rpc_test_setup;
        let server = jsonrpsee::server::Server::builder().build("127.0.0.1:0").await.expect("Starting server");
        let server_url = format!("ws://{}", server.local_addr().expect("Retrieving server local address"));
        // Server will be stopped once this is dropped
        let _server_handle = server.start(StarknetWsRpcApiV0_8_0Server::into_rpc(starknet));
        let client = WsClientBuilder::default().build(&server_url).await.expect("Building client");

        let mut generator = block_generator(&backend);
        let expected = generator.next().expect("Retrieving block from backend");

        let mut sub = client
            .subscribe_new_heads(starknet_core::types::BlockId::Number(0))
            .await
            .expect("starknet_subscribeNewHeads");

        let next = sub.next().await;
        let header = next.expect("Waiting for block header").expect("Waiting for block header");

        assert_eq!(
            header,
            expected,
            "actual: {}\nexpect: {}",
            serde_json::to_string_pretty(&header).unwrap_or_default(),
            serde_json::to_string_pretty(&expected).unwrap_or_default()
        );

        let next = sub.unsubscribe().await;
        assert!(next.is_ok());
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn subscribe_new_heads_future(rpc_test_setup: (std::sync::Arc<mc_db::MadaraBackend>, Starknet)) {
        let (backend, starknet) = rpc_test_setup;
        let server = jsonrpsee::server::Server::builder().build("127.0.0.1:0").await.expect("Starting server");
        let server_url = format!("ws://{}", server.local_addr().expect("Retrieving server local address"));
        // Server will be stopped once this is dropped
        let _server_handle = server.start(StarknetWsRpcApiV0_8_0Server::into_rpc(starknet));
        let client = WsClientBuilder::default().build(&server_url).await.expect("Building client");

        let mut generator = block_generator(&backend);
        let _block_0 = generator.next().expect("Retrieving block from backend");

        let mut sub = client
            .subscribe_new_heads(starknet_core::types::BlockId::Number(1))
            .await
            .expect("starknet_subscribeNewHeads");

        let block_1 = generator.next().expect("Retrieving block from backend");

        let next = sub.next().await;
        let header = next.expect("Waiting for block header").expect("Waiting for block header");

        // Note that `sub` does not yield block 0. This is because it starts
        // from block 1, ignoring any block before. This can server to notify
        // when a block is ready
        assert_eq!(
            header,
            block_1,
            "actual: {}\nexpect: {}",
            serde_json::to_string_pretty(&header).unwrap_or_default(),
            serde_json::to_string_pretty(&block_1).unwrap_or_default()
        );
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn subscribe_new_heads_err_too_far_back_block_n(
        rpc_test_setup: (std::sync::Arc<mc_db::MadaraBackend>, Starknet),
    ) {
        let (backend, starknet) = rpc_test_setup;
        let server = jsonrpsee::server::Server::builder().build("127.0.0.1:0").await.expect("Starting server");
        let server_url = format!("ws://{}", server.local_addr().expect("Retrieving server local address"));
        // Server will be stopped once this is dropped
        let _server_handle = server.start(StarknetWsRpcApiV0_8_0Server::into_rpc(starknet));
        let client = WsClientBuilder::default().build(&server_url).await.expect("Building client");

        // We generate BLOCK_PAST_LIMIT + 2 because genesis is block 0
        let generator = block_generator(&backend);
        let _expected: Vec<_> = generator.take(BLOCK_PAST_LIMIT as usize + 2).collect();

        let mut sub = client
            .subscribe_new_heads(starknet_core::types::BlockId::Number(0))
            .await
            .expect("starknet_subscribeNewHeads");

        // Jsonrsee seems to just close the connection and not return the error
        // to the client so this is the best we can do :/
        let next = sub.next().await;
        assert!(next.is_none());
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn subscribe_new_heads_err_too_far_back_block_hash(
        rpc_test_setup: (std::sync::Arc<mc_db::MadaraBackend>, Starknet),
    ) {
        let (backend, starknet) = rpc_test_setup;
        let server = jsonrpsee::server::Server::builder().build("127.0.0.1:0").await.expect("Starting server");
        let server_url = format!("ws://{}", server.local_addr().expect("Retrieving server local address"));
        // Server will be stopped once this is dropped
        let _server_handle = server.start(StarknetWsRpcApiV0_8_0Server::into_rpc(starknet));
        let client = WsClientBuilder::default().build(&server_url).await.expect("Building client");

        // We generate BLOCK_PAST_LIMIT + 2 because genesis is block 0
        let generator = block_generator(&backend);
        let _expected: Vec<_> = generator.take(BLOCK_PAST_LIMIT as usize + 2).collect();

        let mut sub = client
            .subscribe_new_heads(starknet_core::types::BlockId::Hash(Felt::from(0)))
            .await
            .expect("starknet_subscribeNewHeads");

        // Jsonrsee seems to just close the connection and not return the error
        // to the client so this is the best we can do :/
        let next = sub.next().await;
        assert!(next.is_none());
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn subscribe_new_heads_err_pending(rpc_test_setup: (std::sync::Arc<mc_db::MadaraBackend>, Starknet)) {
        let (backend, starknet) = rpc_test_setup;
        let server = jsonrpsee::server::Server::builder().build("127.0.0.1:0").await.expect("Starting server");
        let server_url = format!("ws://{}", server.local_addr().expect("Retrieving server local address"));
        // Server will be stopped once this is dropped
        let _server_handle = server.start(StarknetWsRpcApiV0_8_0Server::into_rpc(starknet));
        let client = WsClientBuilder::default().build(&server_url).await.expect("Building client");

        let generator = block_generator(&backend);
        let _expected: Vec<_> = generator.take(BLOCK_PAST_LIMIT as usize + 2).collect();

        let mut sub = client
            .subscribe_new_heads(starknet_core::types::BlockId::Tag(starknet_core::types::BlockTag::Pending))
            .await
            .expect("starknet_subscribeNewHeads");

        // Jsonrsee seems to just close the connection and not return the error
        // to the client so this is the best we can do :/
        let next = sub.next().await;
        assert!(next.is_none());
    }
}