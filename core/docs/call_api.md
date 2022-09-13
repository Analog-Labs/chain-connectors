# call_api

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
**call**](call_api.md#call) | **POST** /call | Make a Network-Specific Procedure Call


# **call**
> models::CallResponse call(call_request)
Make a Network-Specific Procedure Call

Call invokes an arbitrary, network-specific procedure call with network-specific parameters. The guidance for what this endpoint should or could do is purposely left vague. In Ethereum, this could be used to invoke `eth_call` to implement an entire Rosetta API interface for some smart contract that is not parsed by the implementation creator (like a DEX). This endpoint could also be used to provide access to data that does not map to any Rosetta models instead of requiring an integrator to use some network-specific SDK and call some network-specific endpoint (like surfacing staking parameters).  Call is NOT a replacement for implementing Rosetta API endpoints or mapping network-specific data to Rosetta models. Rather, it enables developers to build additional Rosetta API interfaces for things they care about without introducing complexity into a base-level Rosetta implementation. Simply put, imagine that the average integrator will use layered Rosetta API implementations that each surfaces unique data. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **call_request** | [**CallRequest**](CallRequest.md)|  | 

### Return type

[**models::CallResponse**](CallResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

