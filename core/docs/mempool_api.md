# mempool_api

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
**mempool**](mempool_api.md#mempool) | **POST** /mempool | Get All Mempool Transactions
**mempoolTransaction**](mempool_api.md#mempoolTransaction) | **POST** /mempool/transaction | Get a Mempool Transaction


# **mempool**
> models::MempoolResponse mempool(network_request)
Get All Mempool Transactions

Get all Transaction Identifiers in the mempool

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **network_request** | [**NetworkRequest**](NetworkRequest.md)|  | 

### Return type

[**models::MempoolResponse**](MempoolResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **mempoolTransaction**
> models::MempoolTransactionResponse mempoolTransaction(mempool_transaction_request)
Get a Mempool Transaction

Get a transaction in the mempool by its Transaction Identifier. This is a separate request than fetching a block transaction (/block/transaction) because some blockchain nodes need to know that a transaction query is for something in the mempool instead of a transaction in a block.  Transactions may not be fully parsable until they are in a block (ex: may not be possible to determine the fee to pay before a transaction is executed). On this endpoint, it is ok that returned transactions are only estimates of what may actually be included in a block. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **mempool_transaction_request** | [**MempoolTransactionRequest**](MempoolTransactionRequest.md)|  | 

### Return type

[**models::MempoolTransactionResponse**](MempoolTransactionResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

