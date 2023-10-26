use alloc::{borrow::Cow, vec::Vec};
use rosetta_ethereum_primitives::{Address, H256, Bytes};

/// Exit reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExecutionResult {
	/// Machine has succeeded.
	Succeed(ExecutionSucceed),
	/// Machine encountered an explicit revert.
	Revert(ExecutionReverted),
	/// Machine returns a normal EVM error.
	Error(ExecutionError),
}

impl ExecutionResult {
	pub const fn output(&self) -> Option<&Bytes> {
		match self {
			Self::Succeed(result) => Some(&result.output),
			Self::Revert(result) => Some(&result.output),
			Self::Error { .. } => None,
		}
	}

	pub fn logs(&self) -> &[Log] {
		match self {
			Self::Succeed(result) => &result.logs,
			Self::Error { .. } | Self::Revert { .. } => &[],
		}
	}

	/// Whether the exit is succeeded.
	pub const fn is_succeed(&self) -> bool {
		matches!(self, Self::Succeed { .. })
	}

	/// Whether the exit is revert.
	pub const fn is_revert(&self) -> bool {
		matches!(self, Self::Revert { .. })
	}

	/// Whether the exit is error.
	pub const fn is_error(&self) -> bool {
		matches!(self, Self::Error { .. })
	}

	/// returns the gas used
	pub const fn gas_used(&self) -> u64 {
		match self {
			Self::Succeed(result) => result.gas_used,
			Self::Revert(result) => result.gas_used,
			Self::Error(result) => result.gas_used,
		}
	}

	/// returns the gas refunded
	pub const fn gas_refunded(&self) -> u64 {
		match self {
			Self::Succeed(result) => result.gas_refunded,
			Self::Revert(_) | Self::Error(_) => 0,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
	feature = "with-codec",
	derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExecutionSucceed {
	pub reason: ExitSucceed,
	pub output: Bytes,
	pub gas_used: u64,
	pub gas_refunded: u64,
	pub logs: Vec<Log>,
}

impl From<ExecutionSucceed> for ExecutionResult {
	fn from(succeed: ExecutionSucceed) -> Self {
		Self::Succeed(succeed)
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
	feature = "with-codec",
	derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExecutionReverted {
	pub output: Bytes,
	pub gas_used: u64,
}

impl From<ExecutionReverted> for ExecutionResult {
	fn from(revert: ExecutionReverted) -> Self {
		Self::Revert(revert)
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
	feature = "with-codec",
	derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExecutionError {
	pub reason: ExitError,
	pub gas_used: u64,
}

impl From<ExecutionError> for ExecutionResult {
	fn from(error: ExecutionError) -> Self {
		Self::Error(error)
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
	feature = "with-codec",
	derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Log {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Bytes,
}

/// Exit succeed reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitSucceed {
	/// Machine encountered an explicit stop.
	Stopped,
	/// Machine encountered an explicit return.
	Returned,
	/// Machine encountered an explicit selfdestruct.
	SelfDestruct,
}

/// Indicates that the EVM has experienced an exceptional halt. This causes execution to
/// immediately end with all gas being consumed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitError {
	/// Trying to pop from an empty stack.
	#[cfg_attr(feature = "with-codec", codec(index = 0))]
	StackUnderflow,
	/// Trying to push into a stack over stack limit.
	#[cfg_attr(feature = "with-codec", codec(index = 1))]
	StackOverflow,
	/// Jump destination is invalid.
	#[cfg_attr(feature = "with-codec", codec(index = 2))]
	InvalidJump,
	/// Call stack is too deep (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 5))]
	CallTooDeep,
	/// Create opcode encountered collision (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 6))]
	CreateCollision,
	/// Create init code exceeds limit (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 7))]
	CreateContractLimit,
	/// Invalid opcode during execution or starting byte is 0xef. See [EIP-3541](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3541.md).
	#[cfg_attr(feature = "with-codec", codec(index = 15))]
	InvalidCode,
	/// An opcode accesses external information, but the request is off offset
	/// limit (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 8))]
	OutOfOffset,
	/// Execution runs out of gas (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 9))]
	OutOfGas,
	/// Not enough fund to start the execution (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 10))]
	OutOfFund,
	/// Other normal errors.
	#[cfg_attr(feature = "with-codec", codec(index = 13))]
	Other(Cow<'static, str>),
	/// Nonce reached maximum value of 2^64-1
	/// https://eips.ethereum.org/EIPS/eip-2681
	#[cfg_attr(feature = "with-codec", codec(index = 14))]
	MaxNonce,
}

