# events_api

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
**eventsBlocks**](events_api.md#eventsBlocks) | **POST** /events/blocks | [INDEXER] Get a range of BlockEvents 


# **eventsBlocks**
> models::EventsBlocksResponse eventsBlocks(events_blocks_request)
[INDEXER] Get a range of BlockEvents 

`/events/blocks` allows the caller to query a sequence of BlockEvents indicating which blocks were added and removed from storage to reach the current state. Following BlockEvents allows lightweight clients to update their state without needing to implement their own syncing logic (like finding the common parent in a reorg).  `/events/blocks` is considered an \"indexer\" endpoint and Rosetta implementations are not required to complete it to adhere to the Rosetta spec. However, any Rosetta \"indexer\" MUST support this endpoint. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **events_blocks_request** | [**EventsBlocksRequest**](EventsBlocksRequest.md)|  | 

### Return type

[**models::EventsBlocksResponse**](EventsBlocksResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

