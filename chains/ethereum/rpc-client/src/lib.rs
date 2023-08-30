mod client;
mod error;
mod params;
mod pubsub;
mod subscription;

pub use client::ClientAdapter;
pub use pubsub::PubsubAdapter;
pub use subscription::SubscriptionStream;

pub mod prelude {
    pub use ethers::providers::{JsonRpcClient, PubsubClient, RpcError};
    pub use jsonrpsee::core::{
        client::{ClientT, SubscriptionClientT},
        traits::ToRpcParams,
    };
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
}
