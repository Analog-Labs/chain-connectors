## __Arbitary Call__

### __Making extrinsic calls__

To make signed extrinsic calls we have combine rosetta-endpoints and construct a pipeline to execute extrinsic.\
`1.` We need to get nonce of user for which we call `contrusction_metadata`.\
`2.` Then we call `construction_payload` endpoint by giving it the `metadata` with following keys.
* `pallet_name`
* `call_name`
* `params`
* `nonce`
* `sender_address`

`3.` After getting the payload we sign the payload from account and call `construction_combine` command to make extrinsic hex\
`4.` We submit extrinsic hex to `construction_submit` endpoint which then returns the transaction hash.

### __Fetching Storage or Constant__

To fetch Storage or any Constant from a Substrate chain, you can use the `Call` method of rosetta. This method takes a `CallRequest` as input and returns a `CallResponse` as output. The `CallRequest` contains the following fields:
* `NetworkIdentifier` - The network to make the call on.
* `Method` - A string contains the name of pallet, function of pallet and type of query e.g. `Storage`, `Constant`. 
* `Params` - A Array containing the parameters required for that pallet function.
