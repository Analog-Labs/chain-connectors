// use parity_scale_codec::{Decode, Encode};
// use rosetta_core::traits::Config;

pub mod queries {
    use parity_scale_codec::{Decode, Encode};
    use rosetta_ethereum_primitives::{
        Address, BlockIdentifier, Bytes, TransactionReceipt, TxHash, U256,
    };
    use serde::{Deserialize, Serialize};

    pub trait EthQuery: Encode + Decode {
        type Result: Encode + Decode;
    }

    /// Parameters for sending a transaction
    #[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
    pub struct Call {
        /// Sender address or ENS name
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from: Option<Address>,

        /// Recipient address (None for contract creation)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub to: Option<Address>,

        /// Transferred value (None for no transfer)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<U256>,

        /// The compiled code of a contract OR the first 4 bytes of the hash of the
        /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<Bytes>,
    }

    ///·Returns·the·balance·of·the·account·of·given·address.
    #[derive(Encode, Decode)]
    pub struct GetBalanceQuery {
        /// Account address
        pub address: Address,
        /// Balance at the block
        pub block: BlockIdentifier,
    }

    impl EthQuery for GetBalanceQuery {
        type Result = U256;
    }

    /// Returns the value from a storage position at a given address.
    #[derive(Encode, Decode)]
    pub struct GetStorageAtQuery {
        /// Account address
        pub address: Address,
        /// integer of the position in the storage.
        pub at: U256,
        /// Storage at the block
        pub block: BlockIdentifier,
    }

    impl EthQuery for GetStorageAtQuery {
        type Result = U256;
    }

    /// Returns the account and storage values of the specified account including the Merkle-proof.
    /// This call can be used to verify that the data you are pulling from is not tampered with.
    #[derive(Encode, Decode)]
    pub struct GetTransactionReceiptQuery {
        tx_hash: TxHash,
    }

    impl EthQuery for GetTransactionReceiptQuery {
        type Result = TransactionReceipt;
    }

    /// Executes a new message call immediately without creating a transaction on the block chain.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    pub struct CallContractQuery {
        /// The address the transaction is sent from.
        from: Option<Address>,
        /// The address the transaction is directed to.
        to: Address,
        /// Integer of the value sent with this transaction.
        value: U256,
        /// Hash of the method signature and encoded parameters.
        data: Bytes,
        /// Call at block
        block: BlockIdentifier,
    }

    /// The result of contract call execution
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    pub enum CallResult {
        /// Call executed succesfully
        Success(Bytes),
        /// Call reverted with message
        Revert(Bytes),
        /// Account doesn't exists
        ContractNotFound,
        /// Out of gas
        OutOfGas,
        /// Call is invalid
        /// Ex: gas price > 64 bits
        InvalidCall,
    }

    impl EthQuery for CallContractQuery {
        type Result = CallResult;
    }
}

pub mod config {
    use parity_scale_codec::{Decode, Encode};
    use rosetta_ethereum_primitives::{Address, Block, BlockIdentifier, TxHash, H256, U256};

    use super::queries::{
        CallContractQuery, EthQuery, GetBalanceQuery, GetStorageAtQuery, GetTransactionReceiptQuery,
    };
    use rosetta_core::traits::Config;

    #[derive(Decode, Encode)]
    pub enum Query {
        /// Returns the balance of the account of given address.
        GetBalance(GetBalanceQuery),
        /// Returns the value from a storage position at a given address.
        GetStorageAt(GetStorageAtQuery),
        /// Returns the receipt of a transaction by transaction hash.
        GetTransactionReceipt(GetTransactionReceiptQuery),
        /// Executes a new message call immediately without creating a transaction on the block
        /// chain.
        CallContract(CallContractQuery),
        /// Returns the account and storage values of the specified account including the
        /// Merkle-proof. This call can be used to verify that the data you are pulling
        /// from is not tampered with.
        GetProof {
            /// Address of the Account
            address: Address,
            /// an array of storage-keys that should be proofed and included
            storage_keys: Vec<U256>,
            /// State at the block
            block: BlockIdentifier,
        },
    }

    #[allow(clippy::large_enum_variant)]
    #[derive(Decode, Encode)]
    pub enum QueryResult {
        /// Returns the balance of the account of given address.
        GetBalance(<GetBalanceQuery as EthQuery>::Result),
        /// Returns the value from a storage position at a given address.
        GetStorageAt(<GetStorageAtQuery as EthQuery>::Result),
        /// Returns the receipt of a transaction by transaction hash.
        GetTransactionReceipt(<GetTransactionReceiptQuery as EthQuery>::Result),
        /// Executes a new message call immediately without creating a transaction on the block
        /// chain.
        CallContract(<CallContractQuery as EthQuery>::Result),
    }

    pub struct EthereumConfig;

    // TODO implement scale codec for primitive types
    impl Config for EthereumConfig {
        type Transaction = ();
        type TransactionIdentifier = TxHash;

        type Block = Block<TxHash>;
        type BlockIdentifier = H256;

        type Query = Query;
        type QueryResult = QueryResult;

        type Event = ();
    }
}
