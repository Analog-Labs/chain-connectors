# AccountCoinsRequest

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**network_identifier** | [***models::NetworkIdentifier**](NetworkIdentifier.md) |  | 
**account_identifier** | [***models::AccountIdentifier**](AccountIdentifier.md) |  | 
**include_mempool** | **bool** | Include state from the mempool when looking up an account's unspent coins. Note, using this functionality breaks any guarantee of idempotency.  | 
**currencies** | [**Vec<models::Currency>**](Currency.md) | In some cases, the caller may not want to retrieve coins for all currencies for an AccountIdentifier. If the currencies field is populated, only coins for the specified currencies will be returned. If not populated, all unspent coins will be returned.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


