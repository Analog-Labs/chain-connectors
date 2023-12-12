use ethereum_types::{Address, Bloom, H256, U256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthereumMetadataParams {
    pub destination: Vec<u8>,
    pub amount: [u64; 4],
    pub data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EthereumMetadata {
    pub chain_id: u64,
    pub nonce: u64,
    pub max_priority_fee_per_gas: [u64; 4],
    pub max_fee_per_gas: [u64; 4],
    pub gas_limit: [u64; 4],
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum AtBlock {
    #[default]
    Latest,
    Hash(H256),
    Number(u64),
}

///·Returns·the·balance·of·the·account·of·given·address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GetBalance {
    /// Account address
    pub address: Address,
    /// Balance at the block
    pub block: AtBlock,
}

/// Executes a new message call immediately without creating a transaction on the blockchain.
#[derive(Clone, Default, PartialEq, Eq, Debug, Hash)]
pub struct CallContract {
    /// The address the transaction is sent from.
    pub from: Option<Address>,
    /// The address the transaction is directed to.
    pub to: Address,
    /// Integer of the value sent with this transaction.
    pub value: U256,
    /// Hash of the method signature and encoded parameters.
    pub data: Vec<u8>,
    /// Call at block
    pub block: AtBlock,
}

/// Returns the account and storage values of the specified account including the Merkle-proof.
/// This call can be used to verify that the data you are pulling from is not tampered with.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GetTransactionReceipt {
    pub tx_hash: H256,
}

/// Returns the value from a storage position at a given address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GetStorageAt {
    /// Account address
    pub address: Address,
    /// integer of the position in the storage.
    pub at: H256,
    /// Storage at the block
    pub block: AtBlock,
}

/// Returns the account and storage values, including the Merkle proof, of the specified
/// account.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GetProof {
    pub account: Address,
    pub storage_keys: Vec<H256>,
    pub block: AtBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// Returns the balance of the account of given address.
    GetBalance(GetBalance),
    /// Returns the value from a storage position at a given address.
    GetStorageAt(GetStorageAt),
    /// Returns the receipt of a transaction by transaction hash.
    GetTransactionReceipt(GetTransactionReceipt),
    /// Executes a new message call immediately without creating a transaction on the block
    /// chain.
    CallContract(CallContract),
    /// Returns the account and storage values of the specified account including the
    /// Merkle-proof. This call can be used to verify that the data you are pulling
    /// from is not tampered with.
    GetProof(GetProof),
}

/// The result of contract call execution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallResult {
    /// Call executed succesfully
    Success(Vec<u8>),
    /// Call reverted with message
    Revert(Vec<u8>),
    /// normal EVM error.
    Error,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryResult {
    /// Returns the balance of the account of given address.
    GetBalance(U256),
    /// Returns the value from a storage position at a given address.
    GetStorageAt(H256),
    /// Returns the receipt of a transaction by transaction hash.
    GetTransactionReceipt(Option<TransactionReceipt>),
    /// Executes a new message call immediately without creating a transaction on the block
    /// chain.
    CallContract(CallResult),
    /// Returns the account and storage values of the specified account including the
    /// Merkle-proof. This call can be used to verify that the data you are pulling
    /// from is not tampered with.
    GetProof(EIP1186ProofResponse),
}

/// A log produced by a transaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Log {
    /// H160. the contract that emitted the log
    pub address: Address,

    /// topics: Array of 0 to 4 32 Bytes of indexed log arguments.
    /// (In solidity: The first topic is the hash of the signature of the event
    /// (e.g. `Deposit(address,bytes32,uint256)`), except you declared the event
    /// with the anonymous specifier.)
    pub topics: Vec<H256>,

    /// Data
    pub data: Vec<u8>,

    /// Block Hash
    pub block_hash: Option<H256>,

    /// Block Number
    pub block_number: Option<u64>,

    /// Transaction Hash
    pub transaction_hash: Option<H256>,

    /// Transaction Index
    pub transaction_index: Option<u64>,

    /// Integer of the log index position in the block. None if it's a pending log.
    pub log_index: Option<U256>,

    /// Integer of the transactions index position log was created from.
    /// None when it's a pending log.
    pub transaction_log_index: Option<U256>,

    /// Log Type
    pub log_type: Option<String>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if it's a valid log.
    pub removed: Option<bool>,
}

/// "Receipt" of an executed transaction: details of its execution.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct TransactionReceipt {
    /// Transaction hash.
    pub transaction_hash: H256,

    /// Index within the block.
    pub transaction_index: u64,

    /// Hash of the block this transaction was included within.
    pub block_hash: Option<H256>,

    /// Number of the block this transaction was included within.
    pub block_number: Option<u64>,

    /// address of the sender.
    pub from: Address,

    // address of the receiver. null when its a contract creation transaction.
    pub to: Option<Address>,

    /// Cumulative gas used within the block after this was executed.
    pub cumulative_gas_used: U256,

    /// Gas used by this transaction alone.
    ///
    /// Gas used is `None` if the the client is running in light client mode.
    pub gas_used: Option<U256>,

    /// Contract address created, or `None` if not a deployment.
    pub contract_address: Option<Address>,

    /// Logs generated within this transaction.
    pub logs: Vec<Log>,

    /// Status: either 1 (success) or 0 (failure). Only present after activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub status_code: Option<u64>,

    /// State root. Only present before activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub state_root: Option<H256>,

    /// Logs bloom
    pub logs_bloom: Bloom,

    /// The price paid post-execution by the transaction (i.e. base fee + priority fee).
    /// Both fields in 1559-style transactions are *maximums* (max fee + max priority fee), the
    /// amount that's actually paid by users can only be determined post-execution
    pub effective_gas_price: Option<U256>,

    /// EIP-2718 transaction type
    pub transaction_type: Option<u64>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct StorageProof {
    pub key: H256,
    pub proof: Vec<Vec<u8>>,
    pub value: U256,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct EIP1186ProofResponse {
    pub address: Address,
    pub balance: U256,
    pub code_hash: H256,
    pub nonce: u64,
    pub storage_hash: H256,
    pub account_proof: Vec<Vec<u8>>,
    pub storage_proof: Vec<StorageProof>,
}
