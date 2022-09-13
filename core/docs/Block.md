# Block

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**block_identifier** | [***models::BlockIdentifier**](BlockIdentifier.md) |  | 
**parent_block_identifier** | [***models::BlockIdentifier**](BlockIdentifier.md) |  | 
**timestamp** | **i64** | The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in milliseconds because some blockchains produce blocks more often than once a second.  | 
**transactions** | [**Vec<models::Transaction>**](Transaction.md) |  | 
**metadata** | [***serde_json::Value**](.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


