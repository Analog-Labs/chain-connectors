# OperationStatus

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | **String** | The status is the network-specific status of the operation.  | 
**successful** | **bool** | An Operation is considered successful if the Operation.Amount should affect the Operation.Account. Some blockchains (like Bitcoin) only include successful operations in blocks but other blockchains (like Ethereum) include unsuccessful operations that incur a fee.  To reconcile the computed balance from the stream of Operations, it is critical to understand which Operation.Status indicate an Operation is successful and should affect an Account.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


