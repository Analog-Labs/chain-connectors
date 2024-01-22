use ethers::types::{Bytes, H160, U256, U64};
use serde::{Deserialize, Serialize};

pub const _CALL_CODE_OP_TYPE: &str = "CALLCODE";
pub const _DELEGATE_CALL_OP_TYPE: &str = "DELEGATECALL";
pub const _STATIC_CALL_OP_TYPE: &str = "STATICCALL";

pub const _TRANSFER_GAS_LIMIT: u64 = 21000;

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
    pub ty: String,
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
            trace_type: trace.ty,
            value: trace.value,
            revert: trace.revert,
            error_message: trace.error_message,
        }
    }
}
