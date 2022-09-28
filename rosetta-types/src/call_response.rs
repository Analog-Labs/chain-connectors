/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// CallResponse : CallResponse contains the result of a `/call` invocation.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CallResponse {
    /// Result contains the result of the `/call` invocation. This result will not be inspected or interpreted by Rosetta tooling and is left to the caller to decode.
    #[serde(rename = "result")]
    pub result: serde_json::Value,
    /// Idempotent indicates that if `/call` is invoked with the same CallRequest again, at any point in time, it will return the same CallResponse.  Integrators may cache the CallResponse if this is set to true to avoid making unnecessary calls to the Rosetta implementation. For this reason, implementers should be very conservative about returning true here or they could cause issues for the caller.
    #[serde(rename = "idempotent")]
    pub idempotent: bool,
}

impl CallResponse {
    /// CallResponse contains the result of a `/call` invocation.
    pub fn new(result: serde_json::Value, idempotent: bool) -> CallResponse {
        CallResponse { result, idempotent }
    }
}
