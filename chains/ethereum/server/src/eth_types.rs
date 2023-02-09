use ethers::types::{Bytes, H160, U256, U64};
use serde::{Deserialize, Serialize};

pub const FEE_OP_TYPE: &str = "FEE";
pub const CALL_OP_TYPE: &str = "CALL";
pub const MINING_REWARD_OP_TYPE: &str = "MINER_REWARD";
pub const UNCLE_REWARD_OP_TYPE: &str = "UNCLE_REWARD";
pub const _CALL_CODE_OP_TYPE: &str = "CALLCODE";
pub const _DELEGATE_CALL_OP_TYPE: &str = "DELEGATECALL";
pub const _STATIC_CALL_OP_TYPE: &str = "STATICCALL";
pub const SELF_DESTRUCT_OP_TYPE: &str = "SELFDESTRUCT";
pub const DESTRUCT_OP_TYPE: &str = "DESTRUCT";

pub const CREATE_OP_TYPE: &str = "CREATE";
pub const CREATE2_OP_TYPE: &str = "CREATE2";

pub const SUCCESS_STATUS: &str = "SUCCESS";
pub const FAILURE_STATUS: &str = "FAILURE";

pub const UNCLE_REWARD_MULTIPLIER: u64 = 32;
pub const MAX_UNCLE_DEPTH: u64 = 8;

pub const GENESIS_BLOCK_INDEX: u64 = 0;
pub const _TRANSFER_GAS_LIMIT: u64 = 21000;

pub const FRONTIER_BLOCK_REWARD: u64 = 5000000000000000000;
pub const BYZANTIUM_BLOCK_REWARD: u64 = 3000000000000000000;
pub const CONSTANTINOPLE_BLOCK_REWARD: u64 = 2000000000000000000;

pub struct ChainConfig {
    pub byzantium_block: u64,
    pub constantinople_block: u64,
}

pub const _MAINNET_CHAIN_CONFIG: ChainConfig = ChainConfig {
    byzantium_block: 4370000,
    constantinople_block: 7280000,
};

pub const TESTNET_CHAIN_CONFIG: ChainConfig = ChainConfig {
    byzantium_block: 0,
    constantinople_block: 0,
};

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
    pub input: Bytes,
    pub output: Bytes,
    pub to: H160,
    #[serde(rename = "type")]
    pub trace_type: String,
    pub value: U256,
    #[serde(default = "revert_default")]
    pub revert: bool,
    #[serde(rename = "error", default = "error_default")]
    pub error_message: String,
    #[serde(default = "calls_default")]
    pub calls: Vec<Trace>,
}

impl Trace {
    pub fn flatten(&self) -> FlattenTrace {
        FlattenTrace {
            from: self.from,
            gas: self.gas,
            gas_used: self.gas_used,
            input: self.input.clone(),
            output: self.output.clone(),
            to: self.to,
            trace_type: self.trace_type.clone(),
            value: self.value,
            revert: self.revert,
            error_message: self.error_message.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct FlattenTrace {
    pub from: H160,
    pub gas: U64,
    pub gas_used: U64,
    pub input: Bytes,
    pub output: Bytes,
    pub to: H160,
    pub trace_type: String,
    pub value: U256,
    pub revert: bool,
    pub error_message: String,
}

fn revert_default() -> bool {
    false
}

fn error_default() -> String {
    "".into()
}

fn calls_default() -> Vec<Trace> {
    vec![]
}
