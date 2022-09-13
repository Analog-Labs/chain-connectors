# Currency

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**symbol** | **String** | Canonical symbol associated with a currency.  | 
**decimals** | **u32** | Number of decimal places in the standard unit representation of the amount.  For example, BTC has 8 decimals. Note that it is not possible to represent the value of some currency in atomic units that is not base 10.  | 
**metadata** | [***serde_json::Value**](.md) | Any additional information related to the currency itself.  For example, it would be useful to populate this object with the contract address of an ERC-20 token.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


