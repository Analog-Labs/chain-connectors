# account_api

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
**accountBalance**](account_api.md#accountBalance) | **POST** /account/balance | Get an Account's Balance
**accountCoins**](account_api.md#accountCoins) | **POST** /account/coins | Get an Account's Unspent Coins


# **accountBalance**
> models::AccountBalanceResponse accountBalance(account_balance_request)
Get an Account's Balance

Get an array of all AccountBalances for an AccountIdentifier and the BlockIdentifier at which the balance lookup was performed. The BlockIdentifier must always be returned because some consumers of account balance data need to know specifically at which block the balance was calculated to compare balances they compute from operations with the balance returned by the node.  It is important to note that making a balance request for an account without populating the SubAccountIdentifier should not result in the balance of all possible SubAccountIdentifiers being returned. Rather, it should result in the balance pertaining to no SubAccountIdentifiers being returned (sometimes called the liquid balance). To get all balances associated with an account, it may be necessary to perform multiple balance requests with unique AccountIdentifiers.  It is also possible to perform a historical balance lookup (if the server supports it) by passing in an optional BlockIdentifier. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **account_balance_request** | [**AccountBalanceRequest**](AccountBalanceRequest.md)|  | 

### Return type

[**models::AccountBalanceResponse**](AccountBalanceResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **accountCoins**
> models::AccountCoinsResponse accountCoins(account_coins_request)
Get an Account's Unspent Coins

Get an array of all unspent coins for an AccountIdentifier and the BlockIdentifier at which the lookup was performed. If your implementation does not support coins (i.e. it is for an account-based blockchain), you do not need to implement this endpoint. If you implementation does support coins (i.e. it is fro a UTXO-based blockchain), you MUST also complete the `/account/balance` endpoint.  It is important to note that making a coins request for an account without populating the SubAccountIdentifier should not result in the coins of all possible SubAccountIdentifiers being returned. Rather, it should result in the coins pertaining to no SubAccountIdentifiers being returned. To get all coins associated with an account, it may be necessary to perform multiple coin requests with unique AccountIdentifiers.  Optionally, an implementation may choose to support updating an AccountIdentifier's unspent coins based on the contents of the mempool. Note, using this functionality breaks any guarantee of idempotency. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **account_coins_request** | [**AccountCoinsRequest**](AccountCoinsRequest.md)|  | 

### Return type

[**models::AccountCoinsResponse**](AccountCoinsResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

