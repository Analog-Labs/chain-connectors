# ConstructionMetadataRequest

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**network_identifier** | [***models::NetworkIdentifier**](NetworkIdentifier.md) |  | 
**options** | [***serde_json::Value**](.md) | Some blockchains require different metadata for different types of transaction construction (ex: delegation versus a transfer). Instead of requiring a blockchain node to return all possible types of metadata for construction (which may require multiple node fetches), the client can populate an options object to limit the metadata returned to only the subset required.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


