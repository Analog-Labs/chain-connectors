# construction_api

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
**constructionCombine**](construction_api.md#constructionCombine) | **POST** /construction/combine | Create Network Transaction from Signatures
**constructionDerive**](construction_api.md#constructionDerive) | **POST** /construction/derive | Derive an AccountIdentifier from a PublicKey
**constructionHash**](construction_api.md#constructionHash) | **POST** /construction/hash | Get the Hash of a Signed Transaction
**constructionMetadata**](construction_api.md#constructionMetadata) | **POST** /construction/metadata | Get Metadata for Transaction Construction
**constructionParse**](construction_api.md#constructionParse) | **POST** /construction/parse | Parse a Transaction
**constructionPayloads**](construction_api.md#constructionPayloads) | **POST** /construction/payloads | Generate an Unsigned Transaction and Signing Payloads
**constructionPreprocess**](construction_api.md#constructionPreprocess) | **POST** /construction/preprocess | Create a Request to Fetch Metadata
**constructionSubmit**](construction_api.md#constructionSubmit) | **POST** /construction/submit | Submit a Signed Transaction


# **constructionCombine**
> models::ConstructionCombineResponse constructionCombine(construction_combine_request)
Create Network Transaction from Signatures

Combine creates a network-specific transaction from an unsigned transaction and an array of provided signatures.  The signed transaction returned from this method will be sent to the `/construction/submit` endpoint by the caller. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **construction_combine_request** | [**ConstructionCombineRequest**](ConstructionCombineRequest.md)|  | 

### Return type

[**models::ConstructionCombineResponse**](ConstructionCombineResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **constructionDerive**
> models::ConstructionDeriveResponse constructionDerive(construction_derive_request)
Derive an AccountIdentifier from a PublicKey

Derive returns the AccountIdentifier associated with a public key.  Blockchains that require an on-chain action to create an account should not implement this method. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **construction_derive_request** | [**ConstructionDeriveRequest**](ConstructionDeriveRequest.md)|  | 

### Return type

[**models::ConstructionDeriveResponse**](ConstructionDeriveResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **constructionHash**
> models::TransactionIdentifierResponse constructionHash(construction_hash_request)
Get the Hash of a Signed Transaction

TransactionHash returns the network-specific transaction hash for a signed transaction. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **construction_hash_request** | [**ConstructionHashRequest**](ConstructionHashRequest.md)|  | 

### Return type

[**models::TransactionIdentifierResponse**](TransactionIdentifierResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **constructionMetadata**
> models::ConstructionMetadataResponse constructionMetadata(construction_metadata_request)
Get Metadata for Transaction Construction

Get any information required to construct a transaction for a specific network. Metadata returned here could be a recent hash to use, an account sequence number, or even arbitrary chain state. The request used when calling this endpoint is created by calling `/construction/preprocess` in an offline environment.  You should NEVER assume that the request sent to this endpoint will be created by the caller or populated with any custom parameters. This must occur in `/construction/preprocess`.  It is important to clarify that this endpoint should not pre-construct any transactions for the client (this should happen in `/construction/payloads`). This endpoint is left purposely unstructured because of the wide scope of metadata that could be required. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **construction_metadata_request** | [**ConstructionMetadataRequest**](ConstructionMetadataRequest.md)|  | 

### Return type

[**models::ConstructionMetadataResponse**](ConstructionMetadataResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **constructionParse**
> models::ConstructionParseResponse constructionParse(construction_parse_request)
Parse a Transaction

Parse is called on both unsigned and signed transactions to understand the intent of the formulated transaction.  This is run as a sanity check before signing (after `/construction/payloads`) and before broadcast (after `/construction/combine`).  

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **construction_parse_request** | [**ConstructionParseRequest**](ConstructionParseRequest.md)|  | 

### Return type

[**models::ConstructionParseResponse**](ConstructionParseResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **constructionPayloads**
> models::ConstructionPayloadsResponse constructionPayloads(construction_payloads_request)
Generate an Unsigned Transaction and Signing Payloads

Payloads is called with an array of operations and the response from `/construction/metadata`. It returns an unsigned transaction blob and a collection of payloads that must be signed by particular AccountIdentifiers using a certain SignatureType.  The array of operations provided in transaction construction often times can not specify all \"effects\" of a transaction (consider invoked transactions in Ethereum). However, they can deterministically specify the \"intent\" of the transaction, which is sufficient for construction. For this reason, parsing the corresponding transaction in the Data API (when it lands on chain) will contain a superset of whatever operations were provided during construction. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **construction_payloads_request** | [**ConstructionPayloadsRequest**](ConstructionPayloadsRequest.md)|  | 

### Return type

[**models::ConstructionPayloadsResponse**](ConstructionPayloadsResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **constructionPreprocess**
> models::ConstructionPreprocessResponse constructionPreprocess(construction_preprocess_request)
Create a Request to Fetch Metadata

Preprocess is called prior to `/construction/payloads` to construct a request for any metadata that is needed for transaction construction given (i.e. account nonce).  The `options` object returned from this endpoint will be sent to the `/construction/metadata` endpoint UNMODIFIED by the caller (in an offline execution environment). If your Construction API implementation has configuration options, they MUST be specified in the `/construction/preprocess` request (in the `metadata` field). 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **construction_preprocess_request** | [**ConstructionPreprocessRequest**](ConstructionPreprocessRequest.md)|  | 

### Return type

[**models::ConstructionPreprocessResponse**](ConstructionPreprocessResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **constructionSubmit**
> models::TransactionIdentifierResponse constructionSubmit(construction_submit_request)
Submit a Signed Transaction

Submit a pre-signed transaction to the node. This call should not block on the transaction being included in a block. Rather, it should return immediately with an indication of whether or not the transaction was included in the mempool.  The transaction submission response should only return a 200 status if the submitted transaction could be included in the mempool. Otherwise, it should return an error. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **construction_submit_request** | [**ConstructionSubmitRequest**](ConstructionSubmitRequest.md)|  | 

### Return type

[**models::TransactionIdentifierResponse**](TransactionIdentifierResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

