# SearchTransactionsRequest

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**network_identifier** | [***models::NetworkIdentifier**](NetworkIdentifier.md) |  | 
**operator** | [***models::Operator**](Operator.md) |  | [optional] [default to None]
**max_block** | **i64** | max_block is the largest block index to consider when searching for transactions. If this field is not populated, the current block is considered the max_block.  If you do not specify a max_block, it is possible a newly synced block will interfere with paginated transaction queries (as the offset could become invalid with newly added rows).  | [optional] [default to None]
**offset** | **i64** | offset is the offset into the query result to start returning transactions.  If any search conditions are changed, the query offset will change and you must restart your search iteration.  | [optional] [default to None]
**limit** | **i64** | limit is the maximum number of transactions to return in one call. The implementation may return <= limit transactions.  | [optional] [default to None]
**transaction_identifier** | [***models::TransactionIdentifier**](TransactionIdentifier.md) |  | [optional] [default to None]
**account_identifier** | [***models::AccountIdentifier**](AccountIdentifier.md) |  | [optional] [default to None]
**coin_identifier** | [***models::CoinIdentifier**](CoinIdentifier.md) |  | [optional] [default to None]
**currency** | [***models::Currency**](Currency.md) |  | [optional] [default to None]
**status** | **String** | status is the network-specific operation type.  | [optional] [default to None]
**r#type** | **String** | type is the network-specific operation type.  | [optional] [default to None]
**address** | **String** | address is AccountIdentifier.Address. This is used to get all transactions related to an AccountIdentifier.Address, regardless of SubAccountIdentifier.  | [optional] [default to None]
**success** | **bool** | success is a synthetic condition populated by parsing network-specific operation statuses (using the mapping provided in `/network/options`).  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


