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

pub const _TRANSFER_GAS_LIMIT: u64 = 21000;

pub const FRONTIER_BLOCK_REWARD: u64 = 5_000_000_000_000_000_000;
pub const BYZANTIUM_BLOCK_REWARD: u64 = 3_000_000_000_000_000_000;
pub const CONSTANTINOPLE_BLOCK_REWARD: u64 = 2_000_000_000_000_000_000;

pub struct ChainConfig {
    pub byzantium_block: u64,
    pub constantinople_block: u64,
}

pub const _MAINNET_CHAIN_CONFIG: ChainConfig =
    ChainConfig { byzantium_block: 4_370_000, constantinople_block: 7_280_000 };

pub const TESTNET_CHAIN_CONFIG: ChainConfig =
    ChainConfig { byzantium_block: 0, constantinople_block: 0 };

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
    #[serde(default)]
    pub revert: bool,
    #[serde(rename = "error", default)]
    pub error_message: String,
    #[serde(default)]
    pub calls: Vec<Trace>,
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

impl From<Trace> for FlattenTrace {
    fn from(trace: Trace) -> Self {
        Self {
            from: trace.from,
            gas: trace.gas,
            gas_used: trace.gas_used,
            input: trace.input,
            output: trace.output,
            to: trace.to,
            trace_type: trace.trace_type,
            value: trace.value,
            revert: trace.revert,
            error_message: trace.error_message,
        }
    }
}
