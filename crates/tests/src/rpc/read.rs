use crate::{MadaraCmd, MadaraCmdBuilder};
use rstest::*;
use starknet_core::types::{
    BlockHashAndNumber, BlockId, BlockStatus, BlockWithReceipts, BlockWithTxHashes, BlockWithTxs,
    CompressedLegacyContractClass, ComputationResources, ContractClass, ContractStorageDiffItem,
    DataAvailabilityResources, DataResources, DeclareTransaction, DeclareTransactionReceipt, DeclareTransactionV0,
    EmittedEvent, EventFilter, EventsPage, ExecutionResources, ExecutionResult, FeePayment, Felt, FunctionCall,
    L1DataAvailabilityMode, L1HandlerTransaction, L1HandlerTransactionReceipt, LegacyEntryPointsByType,
    MaybePendingBlockWithReceipts, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs, MaybePendingStateUpdate,
    PriceUnit, ReceiptBlock, ResourcePrice, StateDiff, StateUpdate, StorageEntry, Transaction,
    TransactionExecutionStatus, TransactionFinalityStatus, TransactionReceipt, TransactionReceiptWithBlockInfo,
    TransactionStatus, TransactionWithReceipt,
};
use starknet_providers::Provider;
use std::env;
use std::fs::{read_dir, File};
use std::io::BufReader;

#[cfg(test)]
mod test_rpc_read_calls {
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    use super::*;
    use rstest::fixture;
    use starknet_core::types::Felt;
    use tokio::sync::OnceCell;

    static MADARA: Lazy<OnceCell<Mutex<MadaraCmd>>> = Lazy::new(|| OnceCell::new());

    async fn setup_madara() -> MadaraCmd {
        let mut madara = MadaraCmdBuilder::new()
            .args(["--network", "sepolia", "--no-sync-polling", "--n-blocks-to-sync", "20", "--no-l1-sync"])
            .run();

        madara.wait_for_ready().await;
        madara.wait_for_sync_to(19).await;
        madara
    }

    async fn get_madara() -> &'static Mutex<MadaraCmd> {
        MADARA.get_or_init(|| async { Mutex::new(setup_madara().await) }).await
    }
    // TODO: make this run once
    // #[fixture]
    // async fn madara() -> MadaraCmd {
    //     let mut madara = MadaraCmdBuilder::new()
    //         .args(["--network", "sepolia", "--no-sync-polling", "--n-blocks-to-sync", "20", "--no-l1-sync"])
    //         .run();
    //
    //     madara.wait_for_ready().await;
    //     madara.wait_for_sync_to(19).await;
    // }

    #[tokio::test]
    async fn test_block_hash_and_number_works() {
        let madara = get_madara().await;

        assert_eq!(
            madara.lock().unwrap().json_rpc().block_hash_and_number().await.unwrap(),
            BlockHashAndNumber {
                // https://sepolia.voyager.online/block/19
                block_hash: Felt::from_hex_unchecked(
                    "0x4177d1ba942a4ab94f86a476c06f0f9e02363ad410cdf177c54064788c9bcb5"
                ),
                block_number: 19
            }
        );
    }

    #[tokio::test]
    async fn test_get_block_txn_count_works() {
        let madara = get_madara().await;
        assert_eq!(madara.lock().unwrap().json_rpc().get_block_transaction_count(BlockId::Number(2)).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_get_block_txn_with_receipts_works() {
        let madara = get_madara().await;

        let block = madara.lock().unwrap().json_rpc().get_block_with_receipts(BlockId::Number(2)).await.unwrap();

        let expected_block = MaybePendingBlockWithReceipts::Block(BlockWithReceipts {
            status: BlockStatus::AcceptedOnL2,
            block_hash: Felt::from_hex_unchecked("0x7a906dfd1ff77a121b8048e6f750cda9e949d341c4487d4c6a449f183f0e61d"),
            parent_hash: Felt::from_hex_unchecked("0x78b67b11f8c23850041e11fb0f3b39db0bcb2c99d756d5a81321d1b483d79f6"),
            block_number: 2,
            new_root: Felt::from_hex_unchecked("0xe005205a1327f3dff98074e528f7b96f30e0624a1dfcf571bdc81948d150a0"),
            timestamp: 1700475581,
            sequencer_address: Felt::from_hex_unchecked(
                "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8",
            ),
            l1_gas_price: ResourcePrice {
                price_in_fri: Felt::from_hex_unchecked("0x0"),
                price_in_wei: Felt::from_hex_unchecked("0x3b9ad016"),
            },
            l1_data_gas_price: ResourcePrice {
                price_in_fri: Felt::from_hex_unchecked("0x1"),
                price_in_wei: Felt::from_hex_unchecked("0x1"),
            },
            l1_da_mode: L1DataAvailabilityMode::Calldata,
            starknet_version: "0.12.3".to_string(),
            transactions: vec![TransactionWithReceipt {
                transaction: Transaction::Declare(DeclareTransaction::V0(DeclareTransactionV0 {
                    transaction_hash: Felt::from_hex_unchecked(
                        "0x701d9adb9c60bc2fd837fe3989e15aeba4be1a6e72bb6f61ffe35a42866c772",
                    ),
                    sender_address: Felt::from_hex_unchecked("0x1"),
                    max_fee: Felt::from_hex_unchecked("0x0"),
                    signature: vec![],
                    class_hash: Felt::from_hex_unchecked(
                        "0x4f23a756b221f8ce46b72e6a6b10ee7ee6cf3b59790e76e02433104f9a8c5d1",
                    ),
                })),
                receipt: {
                    TransactionReceipt::Declare(DeclareTransactionReceipt {
                        transaction_hash: Felt::from_hex_unchecked(
                            "0x701d9adb9c60bc2fd837fe3989e15aeba4be1a6e72bb6f61ffe35a42866c772",
                        ),
                        actual_fee: FeePayment { amount: Felt::from_hex_unchecked("0x0"), unit: PriceUnit::Wei },
                        finality_status: TransactionFinalityStatus::AcceptedOnL2,
                        messages_sent: vec![],
                        events: vec![],
                        execution_resources: ExecutionResources {
                            computation_resources: ComputationResources {
                                steps: 2711,
                                memory_holes: Some(0),
                                range_check_builtin_applications: Some(63),
                                pedersen_builtin_applications: Some(15),
                                poseidon_builtin_applications: None,
                                ec_op_builtin_applications: None,
                                ecdsa_builtin_applications: None,
                                bitwise_builtin_applications: None,
                                keccak_builtin_applications: None,
                                segment_arena_builtin: None,
                            },
                            data_resources: DataResources {
                                data_availability: DataAvailabilityResources { l1_gas: 0, l1_data_gas: 0 },
                            },
                        },
                        execution_result: ExecutionResult::Succeeded,
                    })
                },
            }],
        });
        assert_eq!(block, expected_block);
    }

    #[tokio::test]
    async fn test_get_block_txn_with_tx_hashes_works() {
        let madara = get_madara().await;

        let block = madara.lock().unwrap().json_rpc().get_block_with_tx_hashes(BlockId::Number(2)).await.unwrap();

        let expected_block = MaybePendingBlockWithTxHashes::Block(BlockWithTxHashes {
            status: BlockStatus::AcceptedOnL2,
            block_hash: Felt::from_hex_unchecked("0x7a906dfd1ff77a121b8048e6f750cda9e949d341c4487d4c6a449f183f0e61d"),
            parent_hash: Felt::from_hex_unchecked("0x78b67b11f8c23850041e11fb0f3b39db0bcb2c99d756d5a81321d1b483d79f6"),
            block_number: 2,
            new_root: Felt::from_hex_unchecked("0xe005205a1327f3dff98074e528f7b96f30e0624a1dfcf571bdc81948d150a0"),
            timestamp: 1700475581,
            sequencer_address: Felt::from_hex_unchecked(
                "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8",
            ),
            l1_gas_price: ResourcePrice {
                price_in_fri: Felt::from_hex_unchecked("0x0"),
                price_in_wei: Felt::from_hex_unchecked("0x3b9ad016"),
            },
            l1_data_gas_price: ResourcePrice {
                price_in_fri: Felt::from_hex_unchecked("0x1"),
                price_in_wei: Felt::from_hex_unchecked("0x1"),
            },
            l1_da_mode: L1DataAvailabilityMode::Calldata,
            starknet_version: "0.12.3".to_string(),
            transactions: vec![Felt::from_hex_unchecked(
                "0x701d9adb9c60bc2fd837fe3989e15aeba4be1a6e72bb6f61ffe35a42866c772",
            )],
        });
        assert_eq!(block, expected_block);
    }

    #[tokio::test]
    async fn test_get_block_txn_with_tx_works() {
        let madara = get_madara().await;

        let block = madara.lock().unwrap().json_rpc().get_block_with_txs(BlockId::Number(2)).await.unwrap();

        let expected_block = MaybePendingBlockWithTxs::Block(BlockWithTxs {
            status: BlockStatus::AcceptedOnL2,
            block_hash: Felt::from_hex_unchecked("0x7a906dfd1ff77a121b8048e6f750cda9e949d341c4487d4c6a449f183f0e61d"),
            parent_hash: Felt::from_hex_unchecked("0x78b67b11f8c23850041e11fb0f3b39db0bcb2c99d756d5a81321d1b483d79f6"),
            block_number: 2,
            new_root: Felt::from_hex_unchecked("0xe005205a1327f3dff98074e528f7b96f30e0624a1dfcf571bdc81948d150a0"),
            timestamp: 1700475581,
            sequencer_address: Felt::from_hex_unchecked(
                "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8",
            ),
            l1_gas_price: ResourcePrice {
                price_in_fri: Felt::from_hex_unchecked("0x0"),
                price_in_wei: Felt::from_hex_unchecked("0x3b9ad016"),
            },
            l1_data_gas_price: ResourcePrice {
                price_in_fri: Felt::from_hex_unchecked("0x1"),
                price_in_wei: Felt::from_hex_unchecked("0x1"),
            },
            l1_da_mode: L1DataAvailabilityMode::Calldata,
            starknet_version: "0.12.3".to_string(),
            transactions: vec![Transaction::Declare(DeclareTransaction::V0(DeclareTransactionV0 {
                transaction_hash: Felt::from_hex_unchecked(
                    "0x701d9adb9c60bc2fd837fe3989e15aeba4be1a6e72bb6f61ffe35a42866c772",
                ),
                sender_address: Felt::from_hex_unchecked("0x1"),
                max_fee: Felt::from_hex_unchecked("0x0"),
                signature: vec![],
                class_hash: Felt::from_hex_unchecked(
                    "0x4f23a756b221f8ce46b72e6a6b10ee7ee6cf3b59790e76e02433104f9a8c5d1",
                ),
            }))],
        });
        assert_eq!(block, expected_block);
    }

    #[tokio::test]
    async fn test_get_class_hash_at_works() {
        let madara = get_madara().await;

        let class_hash = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_class_hash_at(
                BlockId::Number(15),
                Felt::from_hex_unchecked("0x04c5772d1914fe6ce891b64eb35bf3522aeae1315647314aac58b01137607f3f"),
            )
            .await
            .unwrap();
        let expected_class_hash =
            Felt::from_hex_unchecked("0xd0e183745e9dae3e4e78a8ffedcce0903fc4900beace4e0abf192d4c202da3");

        assert_eq!(class_hash, expected_class_hash);
    }

    #[tokio::test]
    async fn test_get_nonce_works() {
        let madara = get_madara().await;

        let nonce = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_nonce(
                BlockId::Number(19),
                Felt::from_hex_unchecked("0x0535ca4e1d1be7ec4a88d51a2962cd6c5aea1be96cb2c0b60eb1721dc34f800d"),
            )
            .await
            .unwrap();
        let expected_nonce = Felt::from_hex_unchecked("0x2");

        assert_eq!(nonce, expected_nonce);
    }

    #[tokio::test]
    async fn test_get_txn_by_block_id_and_index_works() {
        let madara = get_madara().await;

        let txn = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_transaction_by_block_id_and_index(BlockId::Number(16), 1)
            .await
            .unwrap();
        let expected_txn = Transaction::L1Handler(L1HandlerTransaction {
            transaction_hash: Felt::from_hex_unchecked(
                "0x68fa87ed202095170a2f551017bf646180f43f4687553dc45e61598349a9a8a",
            ),
            version: Felt::from_hex_unchecked("0x0"),
            nonce: 11,
            contract_address: Felt::from_hex_unchecked(
                "0x4c5772d1914fe6ce891b64eb35bf3522aeae1315647314aac58b01137607f3f",
            ),
            entry_point_selector: Felt::from_hex_unchecked(
                "0x2d757788a8d8d6f21d1cd40bce38a8222d70654214e96ff95d8086e684fbee5",
            ),
            calldata: vec![
                Felt::from_hex_unchecked("0x8453fc6cd1bcfe8d4dfc069c400b433054d47bdc"),
                Felt::from_hex_unchecked("0x70503f026c7af73cfd2b007fe650e8c310256e9674ac4e42797c291edca5e84"),
                Felt::from_hex_unchecked("0x15fb7f9b8c38000"),
                Felt::from_hex_unchecked("0x0"),
            ],
        });

        assert_eq!(txn, expected_txn);
    }

    #[tokio::test]
    async fn test_get_txn_by_hash_works() {
        let madara = get_madara().await;

        let txn = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_transaction_by_hash(Felt::from_hex_unchecked(
                "0x68fa87ed202095170a2f551017bf646180f43f4687553dc45e61598349a9a8a",
            ))
            .await
            .unwrap();
        let expected_txn = Transaction::L1Handler(L1HandlerTransaction {
            transaction_hash: Felt::from_hex_unchecked(
                "0x68fa87ed202095170a2f551017bf646180f43f4687553dc45e61598349a9a8a",
            ),
            version: Felt::from_hex_unchecked("0x0"),
            nonce: 11,
            contract_address: Felt::from_hex_unchecked(
                "0x4c5772d1914fe6ce891b64eb35bf3522aeae1315647314aac58b01137607f3f",
            ),
            entry_point_selector: Felt::from_hex_unchecked(
                "0x2d757788a8d8d6f21d1cd40bce38a8222d70654214e96ff95d8086e684fbee5",
            ),
            calldata: vec![
                Felt::from_hex_unchecked("0x8453fc6cd1bcfe8d4dfc069c400b433054d47bdc"),
                Felt::from_hex_unchecked("0x70503f026c7af73cfd2b007fe650e8c310256e9674ac4e42797c291edca5e84"),
                Felt::from_hex_unchecked("0x15fb7f9b8c38000"),
                Felt::from_hex_unchecked("0x0"),
            ],
        });

        assert_eq!(txn, expected_txn);
    }

    #[tokio::test]
    async fn test_get_txn_receipt_works() {
        let madara = get_madara().await;
        let txn_receipt = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_transaction_receipt(Felt::from_hex_unchecked(
                "0x701d9adb9c60bc2fd837fe3989e15aeba4be1a6e72bb6f61ffe35a42866c772",
            ))
            .await
            .unwrap();
        let expected_txn_receipt = TransactionReceiptWithBlockInfo {
            receipt: TransactionReceipt::Declare(DeclareTransactionReceipt {
                transaction_hash: Felt::from_hex_unchecked(
                    "0x701d9adb9c60bc2fd837fe3989e15aeba4be1a6e72bb6f61ffe35a42866c772",
                ),
                actual_fee: FeePayment { amount: Felt::from_hex_unchecked("0x0"), unit: PriceUnit::Wei },
                finality_status: TransactionFinalityStatus::AcceptedOnL2,
                messages_sent: vec![],
                events: vec![],
                execution_resources: ExecutionResources {
                    computation_resources: ComputationResources {
                        steps: 2711,
                        memory_holes: Some(0),
                        range_check_builtin_applications: Some(63),
                        pedersen_builtin_applications: Some(15),
                        poseidon_builtin_applications: None,
                        ec_op_builtin_applications: None,
                        ecdsa_builtin_applications: None,
                        bitwise_builtin_applications: None,
                        keccak_builtin_applications: None,
                        segment_arena_builtin: None,
                    },
                    data_resources: DataResources {
                        data_availability: DataAvailabilityResources { l1_gas: 0, l1_data_gas: 0 },
                    },
                },
                execution_result: ExecutionResult::Succeeded,
            }),
            block: ReceiptBlock::Block {
                block_hash: Felt::from_hex_unchecked(
                    "0x7a906dfd1ff77a121b8048e6f750cda9e949d341c4487d4c6a449f183f0e61d",
                ),
                block_number: 2,
            },
        };

        assert_eq!(txn_receipt, expected_txn_receipt);
    }

    #[tokio::test]
    async fn test_get_txn_status_works() {
        let madara = get_madara().await;

        let txn_status = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_transaction_status(Felt::from_hex_unchecked(
                "0x68fa87ed202095170a2f551017bf646180f43f4687553dc45e61598349a9a8a",
            ))
            .await
            .unwrap();
        let expected_txn_status = TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Succeeded);

        assert_eq!(txn_status, expected_txn_status);
    }

    #[tokio::test]
    async fn test_get_storage_at_works() {
        let madara = get_madara().await;

        let storage_response = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_storage_at(
                Felt::from_hex_unchecked("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"),
                Felt::from_hex_unchecked("0x0341c1bdfd89f69748aa00b5742b03adbffd79b8e80cab5c50d91cd8c2a79be1"),
                BlockId::Number(12),
            )
            .await
            .unwrap();
        let expected_storage_response = Felt::from_hex_unchecked("0x4574686572");

        assert_eq!(storage_response, expected_storage_response);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_state_update_works() {
        let madara = get_madara().await;

        let state_update = madara.lock().unwrap().json_rpc().get_state_update(BlockId::Number(13)).await.unwrap();
        let expected_state_update = MaybePendingStateUpdate::Update(StateUpdate {
            block_hash: Felt::from_hex_unchecked("0x12e2fe9e5273b777341a372edc56ca0327dc2237232cf2fed6cecc7398ffe9d"),
            old_root: Felt::from_hex_unchecked("0x7b6d0a312a1304bc1f99396c227a3bf062ff390258d2341309b4f60e6520bc9"),
            new_root: Felt::from_hex_unchecked("0x73ef61c78f5bda0bd3ef54d360484d06d32032e3b9287a71e0798526654a733"),
            state_diff: StateDiff {
                storage_diffs: vec![
                    ContractStorageDiffItem {
                        address: Felt::from_hex_unchecked(
                            "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
                        ),
                        storage_entries: vec![
                            StorageEntry {
                                key: Felt::from_hex_unchecked(
                                    "0x110e2f729c9c2b988559994a3daccd838cf52faf88e18101373e67dd061455a",
                                ),
                                value: Felt::from_hex_unchecked("0xe20a99b3d590000"),
                            },
                            StorageEntry {
                                key: Felt::from_hex_unchecked(
                                    "0x6cfd3e69ed325a8ac721ef6c60099111df74d4c17f62221dc847b26c9e9db3d",
                                ),
                                value: Felt::from_hex_unchecked("0x71afd498d0000"),
                            },
                        ],
                    },
                    ContractStorageDiffItem {
                        address: Felt::from_hex_unchecked("0x1"),
                        storage_entries: vec![StorageEntry {
                            key: Felt::from_hex_unchecked("0x3"),
                            value: Felt::from_hex_unchecked(
                                "0x37644818236ee05b7e3b180bed64ea70ee3dd1553ca334a5c2a290ee276f380",
                            ),
                        }],
                    },
                ],
                deprecated_declared_classes: vec![],
                declared_classes: vec![],
                deployed_contracts: vec![],
                replaced_classes: vec![],
                nonces: vec![],
            },
        });

        assert_eq!(state_update, expected_state_update);
    }

    // request:
    /*
        curl --location 'https://free-rpc.nethermind.io/sepolia-juno/' \
        --header 'Content-Type: application/json' \
        --data '{
            "jsonrpc": "2.0",
            "method": "starknet_getEvents",
            "params": {
                "filter": {
                    "from_block": {
                        "block_number": 0
                    },
                    "to_block": {
                        "block_number": 19
                    },
                    "address": "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
                    "keys": [
                        []
                    ],
                    "continuation_token": "",
                    "chunk_size": 2
                }
            },
       "id": 1
    }'

    */
    #[ignore]
    #[tokio::test]
    async fn test_get_events_works() {
        let madara = get_madara().await;

        let events = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_events(
                EventFilter {
                    from_block: Some(BlockId::Number(0)),
                    to_block: Some(BlockId::Number(19)),
                    address: Some(Felt::from_hex_unchecked(
                        "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
                    )),
                    keys: Some(vec![vec![]]),
                },
                None,
                2,
            )
            .await
            .unwrap();
        let expected_events = EventsPage {
            events: vec![
                EmittedEvent {
                    from_address: Felt::from_hex_unchecked(
                        "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
                    ),
                    keys: vec![Felt::from_hex_unchecked(
                        "0x3774b0545aabb37c45c1eddc6a7dae57de498aae6d5e3589e362d4b4323a533",
                    )],
                    data: vec![
                        Felt::from_hex_unchecked("0x43abaa073c768ebf039c0c4f46db9acc39e9ec165690418060a652aab39e7d8"),
                        Felt::from_hex_unchecked("0x43abaa073c768ebf039c0c4f46db9acc39e9ec165690418060a652aab39e7d8"),
                    ],
                    block_hash: Some(Felt::from_hex_unchecked(
                        "0x5c627d4aeb51280058bed93c7889bce78114d63baad1be0f0aeb32496d5f19c",
                    )),
                    block_number: Some(0),
                    transaction_hash: Felt::from_hex_unchecked(
                        "0x1bec64a9f5ff52154b560fd489ae2aabbfcb31062f7ea70c3c674ddf14b0add",
                    ),
                },
                EmittedEvent {
                    from_address: Felt::from_hex_unchecked(
                        "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
                    ),
                    keys: vec![Felt::from_hex_unchecked(
                        "0x4595132f9b33b7077ebf2e7f3eb746a8e0a6d5c337c71cd8f9bf46cac3cfd7",
                    )],
                    data: vec![Felt::from_hex_unchecked(
                        "0x43abaa073c768ebf039c0c4f46db9acc39e9ec165690418060a652aab39e7d8",
                    )],
                    block_hash: Some(Felt::from_hex_unchecked(
                        "0x5c627d4aeb51280058bed93c7889bce78114d63baad1be0f0aeb32496d5f19c",
                    )),
                    block_number: Some(0),
                    transaction_hash: Felt::from_hex_unchecked(
                        "0x1bec64a9f5ff52154b560fd489ae2aabbfcb31062f7ea70c3c674ddf14b0add",
                    ),
                },
            ],
            continuation_token: Some("4-0".to_string()),
        };

        assert_eq!(events, expected_events);
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_works() {
        let madara = get_madara().await;

        let call_response = madara
            .lock()
            .unwrap()
            .json_rpc()
            .call(
                FunctionCall {
                    contract_address: Felt::from_hex_unchecked(
                        "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
                    ),
                    entry_point_selector: Felt::from_hex_unchecked(
                        "0x361458367e696363fbcc70777d07ebbd2394e89fd0adcaf147faccd1d294d60",
                    ),
                    calldata: vec![],
                },
                BlockId::Number(19),
            )
            .await
            .unwrap();
        let expected_call_response = vec![Felt::from_hex_unchecked("0x4574686572")];

        assert_eq!(call_response, expected_call_response);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_class_works() {
        let madara = get_madara().await;

        let call_response = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_class(
                BlockId::Number(12),
                Felt::from_hex_unchecked("0x3131fa018d520a037686ce3efddeab8f28895662f019ca3ca18a626650f7d1e"),
            )
            .await
            .unwrap();
        let file = File::open("src/rpc/class_hash.json").unwrap();
        let reader = BufReader::new(file);

        let expected_call_response: ContractClass = serde_json::from_reader(reader).unwrap();

        assert_eq!(call_response, expected_call_response);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_class_at_works() {
        let madara = get_madara().await;

        let call_response = madara
            .lock()
            .unwrap()
            .json_rpc()
            .get_class_at(
                BlockId::Number(12),
                Felt::from_hex_unchecked("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"),
            )
            .await
            .unwrap();
        let file = File::open("src/rpc/class_at.json").unwrap();
        let reader = BufReader::new(file);

        let expected_call_response: ContractClass = serde_json::from_reader(reader).unwrap();

        assert_eq!(call_response, expected_call_response);
    }
}
