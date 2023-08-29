mod client;
mod error;
mod params;

pub use client::{JsonRpseeClient as Client, SubscriptionStream};

pub mod prelude {
    pub use ethers::providers::{JsonRpcClient, PubsubClient, RpcError};
    pub use jsonrpsee::core::traits::ToRpcParams;
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
}
