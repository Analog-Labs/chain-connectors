/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `EventsBlocksResponse` contains an ordered collection of `BlockEvents` and the max retrievable sequence.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EventsBlocksResponse {
    /// max_sequence is the maximum available sequence number to fetch.
    #[serde(rename = "max_sequence")]
    pub max_sequence: i64,
    /// events is an array of BlockEvents indicating the order to add and remove blocks to maintain a canonical view of blockchain state. Lightweight clients can use this event stream to update state without implementing their own block syncing logic.
    #[serde(rename = "events")]
    pub events: Vec<crate::BlockEvent>,
}

impl EventsBlocksResponse {
    /// `EventsBlocksResponse` contains an ordered collection of `BlockEvents` and the max retrievable sequence.
    #[must_use]
    pub const fn new(max_sequence: i64, events: Vec<crate::BlockEvent>) -> Self {
        Self {
            max_sequence,
            events,
        }
    }
}
