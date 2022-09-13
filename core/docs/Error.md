# Error

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**code** | **u32** | Code is a network-specific error code. If desired, this code can be equivalent to an HTTP status code.  | 
**message** | **String** | Message is a network-specific error message.  The message MUST NOT change for a given code. In particular, this means that any contextual information should be included in the details field.  | 
**description** | **String** | Description allows the implementer to optionally provide additional information about an error. In many cases, the content of this field will be a copy-and-paste from existing developer documentation.  Description can ONLY be populated with generic information about a particular type of error. It MUST NOT be populated with information about a particular instantiation of an error (use `details` for this).  Whereas the content of Error.Message should stay stable across releases, the content of Error.Description will likely change across releases (as implementers improve error documentation). For this reason, the content in this field is not part of any type assertion (unlike Error.Message).  | [optional] [default to None]
**retriable** | **bool** | An error is retriable if the same request may succeed if submitted again.  | 
**details** | [***serde_json::Value**](.md) | Often times it is useful to return context specific to the request that caused the error (i.e. a sample of the stack trace or impacted account) in addition to the standard error message.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


