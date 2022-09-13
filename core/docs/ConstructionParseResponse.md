# ConstructionParseResponse

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operations** | [**Vec<models::Operation>**](Operation.md) |  | 
**signers** | **Vec<String>** | [DEPRECATED by `account_identifier_signers` in `v1.4.4`] All signers (addresses) of a particular transaction. If the transaction is unsigned, it should be empty.  | [optional] [default to None]
**account_identifier_signers** | [**Vec<models::AccountIdentifier>**](AccountIdentifier.md) |  | [optional] [default to None]
**metadata** | [***serde_json::Value**](.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


