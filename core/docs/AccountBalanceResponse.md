# AccountBalanceResponse

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**block_identifier** | [***models::BlockIdentifier**](BlockIdentifier.md) |  | 
**balances** | [**Vec<models::Amount>**](Amount.md) | A single account may have a balance in multiple currencies.  | 
**metadata** | [***serde_json::Value**](.md) | Account-based blockchains that utilize a nonce or sequence number should include that number in the metadata. This number could be unique to the identifier or global across the account address.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


