# SyncStatus

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**current_index** | **i64** | CurrentIndex is the index of the last synced block in the current stage.  This is a separate field from current_block_identifier in NetworkStatusResponse because blocks with indices up to and including the current_index may not yet be queryable by the caller. To reiterate, all indices up to and including current_block_identifier in NetworkStatusResponse must be queryable via the /block endpoint (excluding indices less than oldest_block_identifier).  | [optional] [default to None]
**target_index** | **i64** | TargetIndex is the index of the block that the implementation is attempting to sync to in the current stage.  | [optional] [default to None]
**stage** | **String** | Stage is the phase of the sync process.  | [optional] [default to None]
**synced** | **bool** | synced is a boolean that indicates if an implementation has synced up to the most recent block. If this field is not populated, the caller should rely on a traditional tip timestamp comparison to determine if an implementation is synced.  This field is particularly useful for quiescent blockchains (blocks only produced when there are pending transactions). In these blockchains, the most recent block could have a timestamp far behind the current time but the node could be healthy and at tip.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


