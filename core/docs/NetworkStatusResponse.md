# NetworkStatusResponse

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**current_block_identifier** | [***models::BlockIdentifier**](BlockIdentifier.md) |  | 
**current_block_timestamp** | **i64** | The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in milliseconds because some blockchains produce blocks more often than once a second.  | 
**genesis_block_identifier** | [***models::BlockIdentifier**](BlockIdentifier.md) |  | 
**oldest_block_identifier** | [***models::BlockIdentifier**](BlockIdentifier.md) |  | [optional] [default to None]
**sync_status** | [***models::SyncStatus**](SyncStatus.md) |  | [optional] [default to None]
**peers** | [**Vec<models::Peer>**](Peer.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


