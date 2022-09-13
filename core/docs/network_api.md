# network_api

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
**networkList**](network_api.md#networkList) | **POST** /network/list | Get List of Available Networks
**networkOptions**](network_api.md#networkOptions) | **POST** /network/options | Get Network Options
**networkStatus**](network_api.md#networkStatus) | **POST** /network/status | Get Network Status


# **networkList**
> models::NetworkListResponse networkList(metadata_request)
Get List of Available Networks

This endpoint returns a list of NetworkIdentifiers that the Rosetta server supports. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **metadata_request** | [**MetadataRequest**](MetadataRequest.md)|  | 

### Return type

[**models::NetworkListResponse**](NetworkListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **networkOptions**
> models::NetworkOptionsResponse networkOptions(network_request)
Get Network Options

This endpoint returns the version information and allowed network-specific types for a NetworkIdentifier. Any NetworkIdentifier returned by /network/list should be accessible here.  Because options are retrievable in the context of a NetworkIdentifier, it is possible to define unique options for each network. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **network_request** | [**NetworkRequest**](NetworkRequest.md)|  | 

### Return type

[**models::NetworkOptionsResponse**](NetworkOptionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **networkStatus**
> models::NetworkStatusResponse networkStatus(network_request)
Get Network Status

This endpoint returns the current status of the network requested. Any NetworkIdentifier returned by /network/list should be accessible here. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **network_request** | [**NetworkRequest**](NetworkRequest.md)|  | 

### Return type

[**models::NetworkStatusResponse**](NetworkStatusResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

