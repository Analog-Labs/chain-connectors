/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `EventsBlocksRequest` : `EventsBlocksRequest` is utilized to fetch a sequence of `BlockEvents` indicating which blocks were added and removed from storage to reach the current state.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EventsBlocksRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: crate::NetworkIdentifier,
    /// offset is the offset into the event stream to sync events from. If this field is not populated, we return the limit events backwards from tip. If this is set to 0, we start from the beginning.
    #[serde(rename = "offset", skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    /// limit is the maximum number of events to fetch in one call. The implementation may return <= limit events.
    #[serde(rename = "limit", skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

impl EventsBlocksRequest {
    /// `EventsBlocksRequest` is utilized to fetch a sequence of `BlockEvents` indicating which blocks were added and removed from storage to reach the current state.
    #[must_use] pub fn new(network_identifier: crate::NetworkIdentifier) -> Self {
        Self {
            network_identifier,
            offset: None,
            limit: None,
        }
    }
}
