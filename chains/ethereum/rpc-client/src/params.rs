use jsonrpsee::core::{error::Error as JsonRpseeError, traits::ToRpcParams};
use serde::Serialize;
use serde_json::value::RawValue;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct RpcParams(Option<Box<RawValue>>);

impl RpcParams {
    pub fn from_serializable<T>(params: &T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        let params = serde_json::value::to_raw_value(params)?;
        Ok(Self(Some(params)))
    }
}

impl ToRpcParams for RpcParams {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, JsonRpseeError> {
        Ok(self.0)
    }
}

impl Display for RpcParams {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(v) => Display::fmt(v.as_ref(), f),
            None => f.write_str("null"),
        }
    }
}
