use core::{
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
};
use derivative::Derivative;
pub use parity_scale_codec::{Decode, Encode};

/// Possible transaction statuses returned from our [`TxProgress::next()`] call.
#[derive(Derivative)]
#[derivative(Debug(bound = "ID: core::fmt::Debug"))]
pub enum TxStatus<ID> {
    /// Transaction is part of the future queue.
    Validated,
    /// The transaction has been broadcast to other nodes.
    Broadcasted,
    /// Transaction is no longer in a best block.
    NoLongerInBestBlock,
    /// Transaction has been included in block with given hash.
    InBestBlock(ID),
    /// Transaction has been finalized by a finality-gadget, e.g GRANDPA
    InFinalizedBlock(ID),
    /// Something went wrong in the node.
    Error {
        /// Human readable message; what went wrong.
        message: String,
    },
    /// Transaction is invalid (bad nonce, signature etc).
    Invalid {
        /// Human readable message; why was it invalid.
        message: String,
    },
    /// The transaction was dropped.
    Dropped {
        /// Human readable message; why was it dropped.
        message: String,
    },
}

/// Event generated by the `Client`.
pub enum ClientEvent<T: Client> {
    /// Event generated by the Client
    Notify(<T::Config as Config>::Event),

    /// Informs that the tx status was updated
    TxStatus { id: T::TransactionId, status: TxStatus<T> },

    /// Query Result
    Query { id: T::QueryId, result: Result<<T::Config as Config>::QueryResult, T::Error> },

    /// Close the connection for the given reason.
    Close(T::Error),
}

// Client Primitives
pub trait Config {
    type Transaction: Encode + Decode;
    type TransactionIdentifier: ToString;

    type Block;
    type BlockIdentifier;

    type Query: Encode + Decode;
    type QueryResult: Encode;

    type Event;
}

pub trait Client: Sized {
    type Config: Config;
    type TransactionId;
    type QueryId;
    type Error: Display;

    /// Submits a signed transaction
    /// # Errors
    /// Should return `Err` if the transaction is invalid
    fn submit(
        &mut self,
        tx: <Self::Config as Config>::Transaction,
    ) -> Result<Self::TransactionId, Self::Error>;

    /// Query some read-only data
    /// # Errors
    /// Should return `Err` if the query is invalid
    fn query(
        &mut self,
        query: <Self::Config as Config>::Query,
    ) -> Result<Self::QueryId, Self::Error>;

    /// Should behave like `Stream::poll()`.
    fn poll_next_event(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<ClientEvent<Self>>;
}
