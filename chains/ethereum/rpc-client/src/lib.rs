mod client;
mod error;
mod params;
mod pubsub;
mod subscription;

pub use client::EthClientAdapter;
pub use pubsub::EthPubsubAdapter;
pub use subscription::SubscriptionStream;

pub mod prelude {
    pub use ethers::providers::{JsonRpcClient, PubsubClient, RpcError};
    pub use jsonrpsee::core::{
        client::{ClientT, SubscriptionClientT},
        traits::ToRpcParams,
    };
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
}
