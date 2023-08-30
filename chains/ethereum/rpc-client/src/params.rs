use jsonrpsee::core::{error::Error as JsonRpseeError, traits::ToRpcParams};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::value::RawValue;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct RpcParams(Box<RawValue>);

impl RpcParams {
    pub fn from_serializable<T>(params: &T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        let params = serde_json::value::to_raw_value(params)?;
        Ok(Self(params))
    }

    pub fn deserialize_as<R: DeserializeOwned + Send>(&self) -> Result<R, serde_json::Error> {
        let params = serde_json::value::to_value(&self.0)?;
        serde_json::from_value::<R>(params)
    }
}

impl ToRpcParams for RpcParams {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, JsonRpseeError> {
        Ok(Some(self.0))
    }
}

impl Display for RpcParams {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
