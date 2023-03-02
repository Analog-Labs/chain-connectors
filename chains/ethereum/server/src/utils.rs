use crate::eth_types::{
    FlattenTrace, ResultGethExecTraces, Trace, BYZANTIUM_BLOCK_REWARD, CALL_OP_TYPE,
    CONSTANTINOPLE_BLOCK_REWARD, CREATE2_OP_TYPE, CREATE_OP_TYPE, DESTRUCT_OP_TYPE, FAILURE_STATUS,
    FEE_OP_TYPE, FRONTIER_BLOCK_REWARD, GENESIS_BLOCK_INDEX, MAX_UNCLE_DEPTH,
    MINING_REWARD_OP_TYPE, SELF_DESTRUCT_OP_TYPE, SUCCESS_STATUS, TESTNET_CHAIN_CONFIG,
    UNCLE_REWARD_MULTIPLIER, UNCLE_REWARD_OP_TYPE,
};
use anyhow::{anyhow, bail, Context, Result};
use ethers::{
    abi::{Abi, Detokenize, Function, HumanReadableParser, InvalidOutputType, Token},
    prelude::*,
    utils::to_checksum,
};
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Block, Transaction, TransactionReceipt, H160, H256, U256, U64},
};
use rosetta_server::types as rosetta_types;
use rosetta_server::types::{
    AccountIdentifier, Amount, BlockIdentifier, Currency, Operation, OperationIdentifier,
    PartialBlockIdentifier, TransactionIdentifier,
};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;

pub fn serialize<T: serde::Serialize>(t: &T) -> serde_json::Value {
    serde_json::to_value(t).expect("Types never fail to serialize.")
}

pub async fn get_block(
    block: &PartialBlockIdentifier,
    client: &Provider<Http>,
) -> Result<(Block<Transaction>, Vec<LoadedTransaction>, Vec<Block<H256>>)> {
    let bl_id = if let Some(hash) = block.hash.as_ref() {
        let h256 = H256::from_str(hash)?;
        BlockId::Hash(h256)
    } else if let Some(index) = block.index {
        let ehters_u64 = U64::from(index);
        let bl_number = BlockNumber::Number(ehters_u64);
        BlockId::Number(bl_number)
    } else {
        bail!("Block identifier is not valid");
    };

    let block_eth = client
        .get_block_with_txs(bl_id)
        .await?
        .context("Block not found")?;

    let block_number = block_eth
        .number
        .context("Block number not found in block result")?
        .as_u64();

    let block_author = block_eth
        .author
        .context("Block author not found in block result")?;

    let block_hash = block_eth
        .hash
        .context("Block hash not found in block result")?;

    let block_transactions = block_eth.transactions.clone();

    let uncles = get_uncles(block_number, &block_eth.uncles, client).await?;

    let receipts = get_block_receipts(block_hash, &block_transactions, client).await?;

    let mut traces = vec![];
    let mut add_traces = false;
    if block_number != GENESIS_BLOCK_INDEX {
        add_traces = true;
        let block_traces = get_block_traces(&block_hash, client).await?;
        traces.extend(block_traces.0);
    }

    let mut loaded_transactions = vec![];
    for (idx, transaction) in block_transactions.iter().enumerate() {
        let tx_receipt = &receipts[idx];

        let (fee_amount, fee_burned) =
            estimatate_gas(transaction, tx_receipt, block_eth.base_fee_per_gas)?;

        let mut loaded_tx = LoadedTransaction {
            transaction: transaction.clone(),
            from: transaction.from,
            block_number: U64::from(block_number),
            block_hash,
            fee_amount,
            fee_burned,
            miner: block_author,
            receipt: receipts[idx].clone(),
            trace: None,
        };

        if !add_traces {
            loaded_transactions.push(loaded_tx);
            continue;
        }

        loaded_tx.trace = Some(traces[idx].result.clone());
        loaded_transactions.push(loaded_tx);
    }

    Ok((block_eth, loaded_transactions, uncles))
}

pub async fn get_transaction(
    block_identifier: &BlockIdentifier,
    hash: &str,
    client: &Provider<Http>,
    currency: &Currency,
) -> Result<rosetta_types::Transaction> {
    let tx_hash = H256::from_str(hash)?;
    let transaction = client
        .get_transaction(tx_hash)
        .await?
        .context("Unable to get transaction")?;

    let ehters_u64 = U64::from(block_identifier.index);
    let bl_number = BlockNumber::Number(ehters_u64);
    let block_num = BlockId::Number(bl_number);

    let block = client
        .get_block(block_num)
        .await?
        .context("Block not found")?;

    let block_hash = block.hash.context("Block hash not found")?;
    let block_author = block.author.context("Block author not found")?;

    let tx_receipt = client
        .get_transaction_receipt(tx_hash)
        .await?
        .context("Transaction receipt not found")?;

    if tx_receipt
        .block_hash
        .context("Block hash not found in tx receipt")?
        != block_hash
    {
        bail!("Transaction receipt block hash does not match block hash");
    }

    let mut add_traces = false;
    let mut traces = vec![];
    if ehters_u64 != U64::from(0) {
        add_traces = true;

        let tx_traces = get_transaction_trace(&tx_hash, client).await?;
        traces.push(tx_traces);
    }

    let (fee_amount, fee_burned) =
        estimatate_gas(&transaction, &tx_receipt, block.base_fee_per_gas)?;

    let mut loaded_transaction = LoadedTransaction {
        transaction: transaction.clone(),
        from: transaction.from,
        block_number: ehters_u64,
        block_hash,
        fee_amount,
        fee_burned,
        miner: block_author,
        receipt: tx_receipt,
        trace: None,
    };

    if add_traces {
        loaded_transaction.trace = Some(traces[0].clone());
    }

    let tx = populate_transaction(loaded_transaction, currency).await?;
    Ok(tx)
}

pub async fn populate_transactions(
    block_identifier: &BlockIdentifier,
    block: &Block<Transaction>,
    uncles: Vec<Block<H256>>,
    loaded_transactions: Vec<LoadedTransaction>,
    currency: &Currency,
) -> Result<Vec<rosetta_types::Transaction>> {
    let mut transactions = vec![];
    let miner = block.author.context("Block author not found")?;
    let block_reward_transaction = get_mining_rewards(block_identifier, &miner, &uncles, currency)?;
    transactions.push(block_reward_transaction);

    for tx in loaded_transactions {
        let transaction = populate_transaction(tx, currency).await?;
        transactions.push(transaction);
    }

    Ok(transactions)
}

pub async fn populate_transaction(
    tx: LoadedTransaction,
    currency: &Currency,
) -> Result<rosetta_types::Transaction> {
    let mut operations = vec![];

    let fee_ops = get_fee_operations(&tx, currency)?;
    operations.extend(fee_ops);

    let tx_trace = tx.trace.context("Transaction trace not found")?;
    let traces = flatten_traces(tx_trace.clone())?;

    let traces_ops = get_traces_operations(traces, operations.len() as i64, currency)?;
    operations.extend(traces_ops);

    let transaction = rosetta_types::Transaction {
        transaction_identifier: TransactionIdentifier {
            hash: hex::encode(tx.transaction.hash),
        },
        operations,
        related_transactions: None,
        metadata: Some(json!({
            "gas_limit" : tx.transaction.gas,
            "gas_price": tx.transaction.gas_price,
            "receipt": tx.receipt,
            "trace": tx_trace,
        })),
    };
    Ok(transaction)
}

pub async fn get_uncles(
    block_index: u64,
    uncles: &[H256],
    client: &Provider<Http>,
) -> Result<Vec<Block<H256>>> {
    let mut uncles_data = vec![];
    for (idx, _) in uncles.iter().enumerate() {
        let index = U64::from(idx);
        let uncle_response = client
            .get_uncle(block_index, index)
            .await?
            .context("Uncle block now found")?;
        uncles_data.push(uncle_response);
    }
    Ok(uncles_data)
}

pub async fn get_block_receipts(
    block_hash: H256,
    transactions: &Vec<Transaction>,
    client: &Provider<Http>,
) -> Result<Vec<TransactionReceipt>> {
    let mut receipts = vec![];
    for tx in transactions {
        let tx_hash = tx.hash;
        let receipt = client
            .get_transaction_receipt(tx_hash)
            .await?
            .context("Transaction receipt not found")?;

        if receipt
            .block_hash
            .context("Block hash not found in receipt")?
            != block_hash
        {
            bail!("Receipt's block hash does not match block hash")
        }

        receipts.push(receipt);
    }
    Ok(receipts)
}

pub async fn get_block_traces(
    hash: &H256,
    client: &Provider<Http>,
) -> Result<ResultGethExecTraces> {
    let hash_serialize = serialize(hash);
    let cfg = json!(
        {
            "tracer": "callTracer"
        }
    );

    let traces: ResultGethExecTraces = client
        .request("debug_traceBlockByHash", [hash_serialize, cfg])
        .await?;

    Ok(traces)
}

async fn get_transaction_trace(hash: &H256, client: &Provider<Http>) -> Result<Trace> {
    let hash_serialize = serialize(hash);
    let cfg = json!(
        {
            "tracer": "callTracer"
        }
    );

    let traces: Trace = client
        .request("debug_traceTransaction", [hash_serialize, cfg])
        .await?;

    Ok(traces)
}

pub fn get_fee_operations(tx: &LoadedTransaction, currency: &Currency) -> Result<Vec<Operation>> {
    let miner_earned_reward = if let Some(fee_burned) = tx.fee_burned {
        tx.fee_amount - fee_burned
    } else {
        tx.fee_amount
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
            address: to_checksum(&tx.from, None),
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
            address: to_checksum(&tx.miner, None),
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

    if let Some(fee_burned) = tx.fee_burned {
        let burned_operation = Operation {
            operation_identifier: OperationIdentifier {
                index: 2,
                network_index: None,
            },
            related_operations: None,
            r#type: FEE_OP_TYPE.into(),
            status: Some(SUCCESS_STATUS.into()),
            account: Some(AccountIdentifier {
                address: to_checksum(&tx.from, None),
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

pub fn get_traces_operations(
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

pub fn get_mining_rewards(
    block_identifier: &BlockIdentifier,
    miner: &H160,
    uncles: &Vec<Block<H256>>,
    currency: &Currency,
) -> Result<rosetta_types::Transaction> {
    let mut operations: Vec<Operation> = vec![];
    let mut mining_reward = mining_reward(block_identifier.index);

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
            address: to_checksum(miner, None),
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
        let block_number = block.number.context("Uncle block has no number")?;
        let uncle_block_reward = (block_number + MAX_UNCLE_DEPTH - block_identifier.index)
            * (mining_reward / MAX_UNCLE_DEPTH);

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
            hash: block_identifier.hash.clone(),
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

pub fn estimatate_gas(
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
    let tx_max_priority_fee_per_gas = tx.max_priority_fee_per_gas.unwrap_or(U256::from(0));

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

pub struct LoadedTransaction {
    pub transaction: Transaction,
    pub from: H160,
    pub block_number: U64,
    pub block_hash: H256,
    pub fee_amount: U256,
    pub fee_burned: Option<U256>,
    pub miner: H160,
    pub receipt: TransactionReceipt,
    pub trace: Option<Trace>,
}

#[derive(Debug)]
pub struct EthDetokenizer {
    pub json: String,
}
impl Detokenize for EthDetokenizer {
    fn from_tokens(tokens: Vec<Token>) -> std::result::Result<Self, InvalidOutputType>
    where
        Self: Sized,
    {
        let json = serde_json::to_string(&tokens)
            .map_err(|e| InvalidOutputType(format!("serde json error {e:?}",)))?;
        Ok(EthDetokenizer { json })
    }
}
