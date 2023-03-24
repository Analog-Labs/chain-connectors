use crate::eth_types::{
    FlattenTrace, Trace, BYZANTIUM_BLOCK_REWARD, CALL_OP_TYPE, CONSTANTINOPLE_BLOCK_REWARD,
    CREATE2_OP_TYPE, CREATE_OP_TYPE, DESTRUCT_OP_TYPE, FAILURE_STATUS, FEE_OP_TYPE,
    FRONTIER_BLOCK_REWARD, MAX_UNCLE_DEPTH, MINING_REWARD_OP_TYPE, SELF_DESTRUCT_OP_TYPE,
    SUCCESS_STATUS, TESTNET_CHAIN_CONFIG, UNCLE_REWARD_MULTIPLIER, UNCLE_REWARD_OP_TYPE,
};
use anyhow::{anyhow, bail, Context, Result};
use ethers::{
    abi::{Abi, Function, HumanReadableParser},
    prelude::*,
    utils::to_checksum,
};
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Block, Transaction, TransactionReceipt, H160, H256, U256, U64},
};
use rosetta_server::types as rosetta_types;
use rosetta_server::types::{
    AccountIdentifier, Amount, Currency, Operation, OperationIdentifier, TransactionIdentifier,
};
use rosetta_server::BlockchainConfig;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;

pub async fn get_transaction<T>(
    client: &Provider<Http>,
    config: &BlockchainConfig,
    block: &Block<T>,
    tx: &Transaction,
) -> Result<rosetta_types::Transaction> {
    let tx_receipt = client
        .get_transaction_receipt(tx.hash)
        .await?
        .context("Transaction receipt not found")?;

    if tx_receipt
        .block_hash
        .context("Block hash not found in tx receipt")?
        != block.hash.unwrap()
    {
        bail!("Transaction receipt block hash does not match block hash");
    }

    let currency = config.currency();
    let mut operations = vec![];

    let (fee_amount, fee_burned) = estimatate_gas(tx, &tx_receipt, block.base_fee_per_gas)?;
    let fee_ops = get_fee_operations(
        tx.from,
        block.author.context("block has no author")?,
        fee_amount,
        fee_burned,
        &currency,
    )?;
    operations.extend(fee_ops);

    let tx_trace = if block.number.unwrap().as_u64() != 0 {
        let trace = get_transaction_trace(&tx.hash, client).await?;
        let flattened = flatten_traces(trace.clone())?;
        let trace_ops = get_traces_operations(flattened, operations.len() as i64, &currency)?;
        operations.extend(trace_ops);
        Some(trace)
    } else {
        None
    };

    Ok(rosetta_types::Transaction {
        transaction_identifier: TransactionIdentifier {
            hash: hex::encode(tx.hash),
        },
        operations,
        related_transactions: None,
        metadata: Some(json!({
            "gas_limit" : tx.gas,
            "gas_price": tx.gas_price,
            "receipt": tx_receipt,
            "trace": tx_trace,
        })),
    })
}

pub async fn block_reward_transaction(
    client: &Provider<Http>,
    config: &BlockchainConfig,
    block: &Block<Transaction>,
) -> Result<rosetta_types::Transaction> {
    let block_number = block.number.context("missing block number")?;
    let block_id = BlockId::Number(BlockNumber::Number(block_number));
    let mut uncles = vec![];
    for (i, _) in block.uncles.iter().enumerate() {
        let uncle = client
            .get_uncle(block_id, U64::from(i))
            .await?
            .context("Uncle block now found")?;
        uncles.push(uncle);
    }
    get_mining_rewards(block, &uncles, &config.currency())
}

async fn get_transaction_trace(hash: &H256, client: &Provider<Http>) -> Result<Trace> {
    let params = json!([
        hash,
        {
            "tracer": "callTracer"
        }
    ]);
    let traces: Trace = client.request("debug_traceTransaction", params).await?;
    Ok(traces)
}

fn get_fee_operations(
    from: H160,
    miner: H160,
    fee_amount: U256,
    fee_burned: Option<U256>,
    currency: &Currency,
) -> Result<Vec<Operation>> {
    let miner_earned_reward = if let Some(fee_burned) = fee_burned {
        fee_amount - fee_burned
    } else {
        fee_amount
    };

    let mut operations = vec![];

    let first_op = Operation {
        operation_identifier: OperationIdentifier {
            index: 0,
            network_index: None,
        },
        related_operations: None,
        r#type: FEE_OP_TYPE.into(),
        status: Some(SUCCESS_STATUS.into()),
        account: Some(AccountIdentifier {
            address: to_checksum(&from, None),
            sub_account: None,
            metadata: None,
        }),
        amount: Some(Amount {
            value: format!("-{miner_earned_reward}"),
            currency: currency.clone(),
            metadata: None,
        }),
        coin_change: None,
        metadata: None,
    };

    let second_op = Operation {
        operation_identifier: OperationIdentifier {
            index: 1,
            network_index: None,
        },
        related_operations: Some(vec![OperationIdentifier {
            index: 0,
            network_index: None,
        }]),
        r#type: FEE_OP_TYPE.into(),
        status: Some(SUCCESS_STATUS.into()),
        account: Some(AccountIdentifier {
            address: to_checksum(&miner, None),
            sub_account: None,
            metadata: None,
        }),
        amount: Some(Amount {
            value: format!("{miner_earned_reward}"),
            currency: currency.clone(),
            metadata: None,
        }),
        coin_change: None,
        metadata: None,
    };

    operations.push(first_op);
    operations.push(second_op);

    if let Some(fee_burned) = fee_burned {
        let burned_operation = Operation {
            operation_identifier: OperationIdentifier {
                index: 2,
                network_index: None,
            },
            related_operations: None,
            r#type: FEE_OP_TYPE.into(),
            status: Some(SUCCESS_STATUS.into()),
            account: Some(AccountIdentifier {
                address: to_checksum(&from, None),
                sub_account: None,
                metadata: None,
            }),
            amount: Some(Amount {
                value: format!("-{fee_burned}"),
                currency: currency.clone(),
                metadata: None,
            }),
            coin_change: None,
            metadata: None,
        };

        operations.push(burned_operation);
    }
    Ok(operations)
}

fn get_traces_operations(
    traces: Vec<FlattenTrace>,
    op_len: i64,
    currency: &Currency,
) -> Result<Vec<Operation>> {
    let mut operations: Vec<Operation> = vec![];
    let mut destroyed_accs: HashMap<String, u64> = HashMap::new();

    if traces.is_empty() {
        return Ok(operations);
    }

    for trace in traces {
        let mut metadata: HashMap<String, String> = HashMap::new();
        let mut operation_status = SUCCESS_STATUS;
        if trace.revert {
            operation_status = FAILURE_STATUS;
            metadata.insert("error".into(), trace.error_message);
        }

        let mut zero_value = false;
        if trace.value == U256::from(0) {
            zero_value = true;
        }

        let mut should_add = true;
        if zero_value && trace.trace_type == CALL_OP_TYPE {
            should_add = false;
        }

        let from = to_checksum(&trace.from, None);
        let to = to_checksum(&trace.to, None);

        if should_add {
            let mut from_operation = Operation {
                operation_identifier: OperationIdentifier {
                    index: op_len + operations.len() as i64,
                    network_index: None,
                },
                related_operations: None,
                r#type: trace.trace_type.clone(),
                status: Some(operation_status.into()),
                account: Some(AccountIdentifier {
                    address: from.clone(),
                    sub_account: None,
                    metadata: None,
                }),
                amount: Some(Amount {
                    value: format!("-{}", trace.value),
                    currency: currency.clone(),
                    metadata: None,
                }),
                coin_change: None,
                metadata: None,
            };

            if zero_value {
                from_operation.amount = None;
            } else if let Some(d_from) = destroyed_accs.get(&from) {
                if operation_status == SUCCESS_STATUS {
                    let amount = d_from - trace.value.as_u64();
                    destroyed_accs.insert(from.clone(), amount);
                }
            }

            operations.push(from_operation);
        }

        if trace.trace_type == SELF_DESTRUCT_OP_TYPE {
            //assigning destroyed from to an empty number
            if from == to {
                continue;
            }
        }

        if to.is_empty() {
            continue;
        }

        // If the account is resurrected, we remove it from
        // the destroyed accounts map.
        if trace.trace_type == CREATE_OP_TYPE || trace.trace_type == CREATE2_OP_TYPE {
            destroyed_accs.remove(&to);
        }

        if should_add {
            let last_op_index = operations[operations.len() - 1].operation_identifier.index;
            let mut to_op = Operation {
                operation_identifier: OperationIdentifier {
                    index: last_op_index + 1,
                    network_index: None,
                },
                related_operations: Some(vec![OperationIdentifier {
                    index: last_op_index,
                    network_index: None,
                }]),
                r#type: trace.trace_type,
                status: Some(operation_status.into()),
                account: Some(AccountIdentifier {
                    address: to.clone(),
                    sub_account: None,
                    metadata: None,
                }),
                amount: Some(Amount {
                    value: format!("{}", trace.value),
                    currency: currency.clone(),
                    metadata: None,
                }),
                coin_change: None,
                metadata: None,
            };

            if zero_value {
                to_op.amount = None;
            } else if let Some(d_to) = destroyed_accs.get(&to) {
                if operation_status == SUCCESS_STATUS {
                    let amount = d_to + trace.value.as_u64();
                    destroyed_accs.insert(to.clone(), amount);
                }
            }

            operations.push(to_op);
        }

        for (k, v) in &destroyed_accs {
            if v == &0 {
                continue;
            }

            if v < &0 {
                //throw some error
            }

            let operation = Operation {
                operation_identifier: OperationIdentifier {
                    index: operations[operations.len() - 1].operation_identifier.index + 1,
                    network_index: None,
                },
                related_operations: None,
                r#type: DESTRUCT_OP_TYPE.into(),
                status: Some(SUCCESS_STATUS.into()),
                account: Some(AccountIdentifier {
                    address: to_checksum(&H160::from_str(k)?, None),
                    sub_account: None,
                    metadata: None,
                }),
                amount: Some(Amount {
                    value: format!("-{v}"),
                    currency: currency.clone(),
                    metadata: None,
                }),
                coin_change: None,
                metadata: None,
            };

            operations.push(operation);
        }
    }

    Ok(operations)
}

fn flatten_traces(data: Trace) -> Result<Vec<FlattenTrace>> {
    let mut traces = vec![];

    let flatten_trace = data.flatten();
    traces.push(flatten_trace);

    for mut child in data.calls {
        if data.revert {
            child.revert = true;

            if child.error_message.is_empty() {
                child.error_message = data.error_message.clone();
            }
        }

        let children = flatten_traces(child)?;
        traces.extend(children);
    }

    Ok(traces)
}

fn get_mining_rewards(
    block: &Block<Transaction>,
    uncles: &Vec<Block<H256>>,
    currency: &Currency,
) -> Result<rosetta_types::Transaction> {
    let block_number = block.number.unwrap().as_u64();
    let block_hash = block.hash.unwrap();
    let miner = block.author.unwrap();
    let mut operations: Vec<Operation> = vec![];
    let mut mining_reward = mining_reward(block_number);

    if !uncles.is_empty() {
        let uncle_reward = (mining_reward / UNCLE_REWARD_MULTIPLIER) as f64;
        let reward_float = uncle_reward * mining_reward as f64;
        let reward_int = reward_float as u64;
        mining_reward += reward_int;
    }

    let mining_reward_operation = Operation {
        operation_identifier: OperationIdentifier {
            index: 0,
            network_index: None,
        },
        related_operations: None,
        r#type: MINING_REWARD_OP_TYPE.into(),
        status: Some(SUCCESS_STATUS.into()),
        account: Some(AccountIdentifier {
            address: to_checksum(&miner, None),
            sub_account: None,
            metadata: None,
        }),
        amount: Some(Amount {
            value: mining_reward.to_string(),
            currency: currency.clone(),
            metadata: None,
        }),
        coin_change: None,
        metadata: None,
    };

    operations.push(mining_reward_operation);

    for block in uncles {
        let uncle_miner = block.author.context("Uncle block has no author")?;
        let uncle_number = block.number.context("Uncle block has no number")?;
        let uncle_block_reward =
            (uncle_number + MAX_UNCLE_DEPTH - block_number) * (mining_reward / MAX_UNCLE_DEPTH);

        let operation = Operation {
            operation_identifier: OperationIdentifier {
                index: operations.len() as i64,
                network_index: None,
            },
            related_operations: None,
            r#type: UNCLE_REWARD_OP_TYPE.into(),
            status: Some(SUCCESS_STATUS.into()),
            account: Some(AccountIdentifier {
                address: to_checksum(&uncle_miner, None),
                sub_account: None,
                metadata: None,
            }),
            amount: Some(Amount {
                value: uncle_block_reward.to_string(),
                currency: currency.clone(),
                metadata: None,
            }),
            coin_change: None,
            metadata: None,
        };

        operations.push(operation);
    }

    Ok(rosetta_types::Transaction {
        transaction_identifier: TransactionIdentifier {
            hash: hex::encode(block_hash),
        },
        related_transactions: None,
        operations,
        metadata: None,
    })
}

fn mining_reward(block_index: u64) -> u64 {
    let mut block_reward = FRONTIER_BLOCK_REWARD;
    if is_byzantium(block_index) {
        block_reward = BYZANTIUM_BLOCK_REWARD;
    };
    if is_constantinopl(block_index) {
        block_reward = CONSTANTINOPLE_BLOCK_REWARD;
    };
    block_reward
}

fn is_byzantium(block_index: u64) -> bool {
    let testnet_config = TESTNET_CHAIN_CONFIG;
    is_block_forked(Some(testnet_config.byzantium_block), Some(block_index))
}

fn is_constantinopl(block_index: u64) -> bool {
    let testnet_config = TESTNET_CHAIN_CONFIG;
    is_block_forked(Some(testnet_config.constantinople_block), Some(block_index))
}

fn is_block_forked(block_val: Option<u64>, head: Option<u64>) -> bool {
    if let Some(bl_val) = block_val {
        if let Some(head_val) = head {
            bl_val <= head_val
        } else {
            false
        }
    } else {
        false
    }
}

fn estimatate_gas(
    tx: &Transaction,
    receipt: &TransactionReceipt,
    base_fee: Option<U256>,
) -> Result<(U256, Option<U256>)> {
    let gas_used = receipt.gas_used.context("gas used is not available")?;
    let gas_price = effective_gas_price(tx, base_fee)?;

    let fee_amount = gas_used * gas_price;

    let fee_burned = base_fee.map(|fee| gas_used * fee);

    Ok((fee_amount, fee_burned))
}

fn effective_gas_price(tx: &Transaction, base_fee: Option<U256>) -> Result<U256> {
    let base_fee = base_fee.context("base fee is not available")?;
    let tx_transaction_type = tx
        .transaction_type
        .context("transaction type is not available")?;
    let tx_gas_price = tx.gas_price.context("gas price is not available")?;
    let tx_max_priority_fee_per_gas = tx.max_priority_fee_per_gas.unwrap_or_default();

    if tx_transaction_type.as_u64() != 2 {
        return Ok(tx_gas_price);
    }

    Ok(base_fee + tx_max_priority_fee_per_gas)
}

pub fn parse_method(method: &str) -> Result<Function> {
    let parse_result = HumanReadableParser::parse_function(method);
    if parse_result.is_ok() {
        parse_result.map_err(|e| anyhow!(e))
    } else {
        let json_parse: Result<Abi, serde_json::Error> =
            if !(method.starts_with('[') && method.ends_with(']')) {
                let abi_str = format!("[{method}]");
                serde_json::from_str(&abi_str)
            } else {
                serde_json::from_str(method)
            };
        let abi: Abi = json_parse?;
        let (_, functions): (&String, &Vec<Function>) = abi
            .functions
            .iter()
            .next()
            .context("No functions found in abi")?;
        let function: Function = functions
            .get(0)
            .context("Abi function list is empty")?
            .clone();
        Ok(function)
    }
}
