use std::fs;

use ethers::{
    providers::{Http, Provider},
    types::{Transaction, TransactionReceipt, H160, H256, U256, U64},
};
use rosetta_server::{
    types::{AccountIdentifier, Amount, Operation, OperationIdentifier},
    BlockchainConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn serialize<T: serde::Serialize>(t: &T) -> serde_json::Value {
    serde_json::to_value(t).expect("Types never fail to serialize.")
}

pub async fn get_block_traces(hash: &H256, client: &Provider<Http>) -> ResultGethExecTraces {
    let hash_serialize = serialize(hash);
    let cfg = json!({ "tracer": "callTracer" });

    // let file = fs::read_to_string("chains/ethereum/server/src/call_tracer.js").unwrap();
    // let cfg = json!({"tracer": file});

    let traces: ResultGethExecTraces = client
        .request("debug_traceBlockByHash", [hash_serialize, cfg])
        .await
        .unwrap();

    traces
}

pub fn get_fee_operations(tx: &LoadedTransaction, config: &BlockchainConfig) -> Vec<Operation> {
    let minerEarnedReward = if let Some(fee_burned) = tx.fee_burned {
        tx.fee_amount - fee_burned
    } else {
        tx.fee_amount
    };

    let mut operations_vec = vec![];

    let first_op = Operation {
        operation_identifier: OperationIdentifier {
            index: 0,
            network_index: None,
        },
        related_operations: Some(vec![]),
        r#type: "FEE".into(),
        status: Some("SUCCESS".into()),
        account: Some(AccountIdentifier {
            address: tx.from.to_string(),
            sub_account: None,
            metadata: None,
        }),
        amount: Some(Amount {
            value: format!("-{}", minerEarnedReward),
            currency: config.currency(),
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
        r#type: "FEE".into(),
        status: Some("SUCCESS".into()),
        account: Some(AccountIdentifier {
            address: tx.miner.to_string(),
            sub_account: None,
            metadata: None,
        }),
        amount: Some(Amount {
            value: format!("{}", minerEarnedReward),
            currency: config.currency(),
            metadata: None,
        }),
        coin_change: None,
        metadata: None,
    };

    operations_vec.push(first_op);
    operations_vec.push(second_op);

    if let Some(fee_burned) = tx.fee_burned {
        let burned_operation = Operation {
            operation_identifier: OperationIdentifier {
                index: 2,
                network_index: None,
            },
            related_operations: Some(vec![]),
            r#type: "FEE".into(),
            status: Some("SUCCESS".into()),
            account: Some(AccountIdentifier {
                address: tx.from.to_string(),
                sub_account: None,
                metadata: None,
            }),
            amount: Some(Amount {
                value: format!("-{}", fee_burned),
                currency: config.currency(),
                metadata: None,
            }),
            coin_change: None,
            metadata: None,
        };

        operations_vec.push(burned_operation);
        operations_vec
    } else {
        return operations_vec;
    }
}

pub fn get_traces_operations(
    tx: &LoadedTransaction,
    traces: &ResultGethExecTrace,
    config: &BlockchainConfig,
) -> Vec<Operation> {
    let mut operations: Vec<Operation> = vec![];



    operations
}

#[derive(Serialize)]
#[doc(hidden)]
pub(crate) struct GethLoggerConfig {
    /// enable memory capture
    #[serde(rename = "EnableMemory")]
    enable_memory: bool,
    /// disable stack capture
    #[serde(rename = "DisableStack")]
    disable_stack: bool,
    /// disable storage capture
    #[serde(rename = "DisableStorage")]
    disable_storage: bool,
    /// enable return data capture
    #[serde(rename = "EnableReturnData")]
    enable_return_data: bool,
}

impl Default for GethLoggerConfig {
    fn default() -> Self {
        Self {
            enable_memory: false,
            disable_stack: false,
            disable_storage: false,
            enable_return_data: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[doc(hidden)]
pub struct ResultGethExecTraces(pub Vec<ResultGethExecTrace>);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[doc(hidden)]
pub struct ResultGethExecTrace {
    pub result: Trace,
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Trace {
    pub from: H160,
    pub gas: U64,
    #[serde(rename = "gasUsed")]
    pub gas_used: U64,
    pub input: String,
    pub output: String,
    pub to: H160,
    #[serde(rename = "type")]
    pub trace_type: String,
    pub value: U256,
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct GethExecTrace {
    /// Used gas
    pub gas: Gas,
    /// True when the transaction has failed.
    pub failed: bool,
    /// Return value of execution which is a hex encoded byte array
    #[serde(rename = "returnValue")]
    pub return_value: String,
    /// Vector of geth execution steps of the trace.
    #[serde(rename = "structLogs")]
    pub struct_logs: Vec<Value>,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Gas(pub u64);

pub struct LoadedTransaction {
    pub transaction: Transaction,
    pub from: H160,
    pub block_number: U64,
    pub block_hash: H256,
    pub fee_amount: U256,
    pub fee_burned: Option<U256>,
    pub miner: H160,
    pub receipt: TransactionReceipt,
}
