// use parity_scale_codec::{Decode, Encode};
// use rosetta_core::traits::Config;

pub mod primitives {
    extern crate alloc;
    use std::{
        borrow::Borrow,
        fmt::{Debug, Display, Formatter, LowerHex, Result as FmtResult},
        ops::Deref,
        str::FromStr,
    };

    use const_hex as hex;
    pub use ethereum::{
        Block, Header, PartialHeader, ReceiptAny as TransactionReceipt, TransactionAny,
    };
    pub use ethereum_types::{Address, H256, U256, U64};
    use parity_scale_codec::{Decode, Encode};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use thiserror::Error;

    /// Wrapper type around Bytes to deserialize/serialize "0x" prefixed ethereum hex strings
    #[derive(
        Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd, Encode, Decode,
    )]
    pub struct Bytes(
        #[serde(serialize_with = "serialize_bytes", deserialize_with = "deserialize_bytes")]
        pub  bytes::Bytes,
    );

    impl hex::FromHex for Bytes {
        type Error = hex::FromHexError;

        fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
            hex::decode(hex).map(Into::into)
        }
    }

    impl FromIterator<u8> for Bytes {
        fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
            iter.into_iter().collect::<bytes::Bytes>().into()
        }
    }

    impl<'a> FromIterator<&'a u8> for Bytes {
        fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
            iter.into_iter().copied().collect::<bytes::Bytes>().into()
        }
    }

    impl Bytes {
        /// Creates a new empty `Bytes`.
        ///
        /// This will not allocate and the returned `Bytes` handle will be empty.
        ///
        /// # Examples
        ///
        /// ```
        /// use ethers_core::types::Bytes;
        ///
        /// let b = Bytes::new();
        /// assert_eq!(&b[..], b"");
        /// ```
        #[inline]
        #[must_use]
        pub const fn new() -> Self {
            Self(bytes::Bytes::new())
        }

        /// Creates a new `Bytes` from a static slice.
        ///
        /// The returned `Bytes` will point directly to the static slice. There is
        /// no allocating or copying.
        ///
        /// # Examples
        ///
        /// ```
        /// use ethers_core::types::Bytes;
        ///
        /// let b = Bytes::from_static(b"hello");
        /// assert_eq!(&b[..], b"hello");
        /// ```
        #[inline]
        #[must_use]
        pub const fn from_static(bytes: &'static [u8]) -> Self {
            Self(bytes::Bytes::from_static(bytes))
        }

        fn hex_encode(&self) -> String {
            hex::encode(self.0.as_ref())
        }
    }

    impl Debug for Bytes {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            write!(f, "Bytes(0x{})", self.hex_encode())
        }
    }

    impl Display for Bytes {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            write!(f, "0x{}", self.hex_encode())
        }
    }

    impl LowerHex for Bytes {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            write!(f, "0x{}", self.hex_encode())
        }
    }

    impl Deref for Bytes {
        type Target = [u8];

        #[inline]
        fn deref(&self) -> &[u8] {
            self.as_ref()
        }
    }

    impl AsRef<[u8]> for Bytes {
        fn as_ref(&self) -> &[u8] {
            self.0.as_ref()
        }
    }

    impl Borrow<[u8]> for Bytes {
        fn borrow(&self) -> &[u8] {
            self.as_ref()
        }
    }

    impl IntoIterator for Bytes {
        type Item = u8;
        type IntoIter = bytes::buf::IntoIter<bytes::Bytes>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl<'a> IntoIterator for &'a Bytes {
        type Item = &'a u8;
        type IntoIter = core::slice::Iter<'a, u8>;

        fn into_iter(self) -> Self::IntoIter {
            self.as_ref().iter()
        }
    }

    impl From<bytes::Bytes> for Bytes {
        fn from(src: bytes::Bytes) -> Self {
            Self(src)
        }
    }

    impl From<Vec<u8>> for Bytes {
        fn from(src: Vec<u8>) -> Self {
            Self(src.into())
        }
    }

    impl<const N: usize> From<[u8; N]> for Bytes {
        fn from(src: [u8; N]) -> Self {
            src.to_vec().into()
        }
    }

    impl<'a, const N: usize> From<&'a [u8; N]> for Bytes {
        fn from(src: &'a [u8; N]) -> Self {
            src.to_vec().into()
        }
    }

    impl PartialEq<[u8]> for Bytes {
        fn eq(&self, other: &[u8]) -> bool {
            self.as_ref() == other
        }
    }

    impl PartialEq<Bytes> for [u8] {
        fn eq(&self, other: &Bytes) -> bool {
            *other == *self
        }
    }

    impl PartialEq<Vec<u8>> for Bytes {
        fn eq(&self, other: &Vec<u8>) -> bool {
            self.as_ref() == &other[..]
        }
    }

    impl PartialEq<Bytes> for Vec<u8> {
        fn eq(&self, other: &Bytes) -> bool {
            *other == *self
        }
    }

    impl PartialEq<bytes::Bytes> for Bytes {
        fn eq(&self, other: &bytes::Bytes) -> bool {
            other == self.as_ref()
        }
    }

    #[derive(Debug, Clone, Error)]
    #[error("Failed to parse bytes: {0}")]
    pub struct ParseBytesError(hex::FromHexError);

    impl FromStr for Bytes {
        type Err = ParseBytesError;

        fn from_str(value: &str) -> Result<Self, Self::Err> {
            hex::FromHex::from_hex(value).map_err(ParseBytesError)
        }
    }

    /// Serialize bytes as "0x" prefixed hex string
    ///
    /// # Errors
    /// never fails
    pub fn serialize_bytes<S, T>(x: T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        s.serialize_str(&hex::encode_prefixed(x))
    }

    /// Deseerialize bytes as "0x" prefixed hex string
    ///
    /// # Errors
    /// never fails
    pub fn deserialize_bytes<'de, D>(d: D) -> Result<bytes::Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(d)?;
        hex::decode(value).map(Into::into).map_err(serde::de::Error::custom)
    }

    pub type TxHash = H256;

    #[derive(Clone, Encode, Decode, PartialEq, Eq, Debug)]
    pub enum BlockIdentifier {
        Hash(H256),
        Number(U64),
    }

    impl Serialize for BlockIdentifier {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::ser::Serializer,
        {
            match self {
                Self::Hash(hash) => <H256 as Serialize>::serialize(hash, serializer),
                Self::Number(number) => <U64 as Serialize>::serialize(number, serializer),
            }
        }
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

        /// Supplied gas (None for sensible default)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub gas: Option<U256>,

        /// Gas price (None for sensible default)
        #[serde(rename = "gasPrice")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub gas_price: Option<U256>,

        /// Transferred value (None for no transfer)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<U256>,

        /// The compiled code of a contract OR the first 4 bytes of the hash of the
        /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<Bytes>,
    }
}

pub mod queries {
    use super::primitives::{Address, BlockIdentifier, Bytes, TransactionReceipt, TxHash, U256};
    use parity_scale_codec::{Decode, Encode};

    pub trait EthQuery: Encode + Decode {
        type Result: Encode + Decode;
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

    use super::{
        primitives::{Address, Block, BlockIdentifier, TxHash, H256, U256},
        queries::{
            CallContractQuery, EthQuery, GetBalanceQuery, GetStorageAtQuery,
            GetTransactionReceiptQuery,
        },
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
