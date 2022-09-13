# EventsBlocksResponse

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max_sequence** | **i64** | max_sequence is the maximum available sequence number to fetch.  | 
**events** | [**Vec<models::BlockEvent>**](BlockEvent.md) | events is an array of BlockEvents indicating the order to add and remove blocks to maintain a canonical view of blockchain state. Lightweight clients can use this event stream to update state without implementing their own block syncing logic.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


