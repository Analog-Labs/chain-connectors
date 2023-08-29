mod client;
mod error;
mod params;
mod subscription;

pub use client::EthClient;
pub use subscription::SubscriptionStream;

pub mod prelude {
    pub use ethers::providers::{JsonRpcClient, PubsubClient, RpcError};
    pub use jsonrpsee::core::traits::ToRpcParams;
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
}
