# SearchTransactionsResponse

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**transactions** | [**Vec<models::BlockTransaction>**](BlockTransaction.md) | transactions is an array of BlockTransactions sorted by most recent BlockIdentifier (meaning that transactions in recent blocks appear first).  If there are many transactions for a particular search, transactions may not contain all matching transactions. It is up to the caller to paginate these transactions using the max_block field.  | 
**total_count** | **i64** | total_count is the number of results for a given search. Callers typically use this value to concurrently fetch results by offset or to display a virtual page number associated with results.  | 
**next_offset** | **i64** | next_offset is the next offset to use when paginating through transaction results. If this field is not populated, there are no more transactions to query.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


