mod client;
mod error;
mod params;
mod pubsub;
mod subscription;

// Re-exports
pub mod exports {
    pub use ethers;
    pub use jsonrpsee;
    pub use serde;
    pub use serde_json;
}

// Adapters
pub use client::EthClientAdapter;
pub use pubsub::EthPubsubAdapter;
pub use subscription::EthSubscription;

/// Easy imports of frequently used traits.
pub mod prelude {
    pub use ethers::providers::{JsonRpcClient, PubsubClient, RpcError};
    pub use jsonrpsee::core::{
        client::{ClientT, SubscriptionClientT},
        traits::ToRpcParams,
    };
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
}
