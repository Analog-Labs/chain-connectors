# __Rosetta Server for Ethereum Chains__

This Project contains `BlockchainClient` implementation of ethereum chains.

Methods implemented are:
* `config`
* `genesis_block`
* `node_version`
* `current_block`
* `balance`
* `faucet`
* `metadata`
* `submit`
* `block`
* `block_transaction`
* `call`


### __`config`__:
This method returns `BlockchainConfig` which contains the configuration specific details for ethereum chain.

### __`genesis_block`__:
Returns genesis block identifier.

### __`node_version`__:
Returns node client version.

### __`current_block`__:
Fetches current block using RPC and returns its identifier.

### __`balance`__:
Fetches account balance from on chain and returns it. It takes two arguments:
`address`: Address of account we want to fetch balance of.
`block`: block identifier of block at which we want to fetch balance of account.

### __`block`__:
This function takes `PartialBlockIdentifier` which contains a block index or hash and returns block transaction and operations happened in that transaction.

### __`block_transaction`__:
This function takes: 
`block`: Which is a block identifier of block from which we want to fetch transaction from.
`tx`: Transaction identifier of transaction we want to fetch.

And returns a specific transaction and its operations within specified block.

### __`faucet`__:

This method is used to fund an account with some amount of tokens in testnet. It takes two arguments:
`address`: Address of account we want to fund.
`amount`: Amount of tokens we want to fund.

### __`metadata`__:

This call is used to fetch nonce of account, It takes two arguments:
`public_key`: This is the public key of sender.
`options`: This is Params needed to create metadata. For ethereum chain it takes
    `destination`: Address of receivier.
    `amount`: Amount to be transfered to receiver.
    `data`: encoded input data for call

It returns `EthereumMetadata` which includes `chain_id`, `nonce` and gas details for transaction.

### __`submit`__:

It takes transaction bytes which is signed transaction bytes and it Submits signed transaction to chain and return its transaction id.

### __`call`__:

This function takes `CallRequest` which contains `method` and `parameters` and returns value returned by function or value stored at specific position in storage or proof of value stored at specific position in storage.

`method`: its a string containing 3 values separated by `-` (dash). <br/>
    1. `contract_address`: This is the name of the contract. <br/>
    2. `method_signature` in case of contract call or `position` in case of storage call. <br/>
    3. `call_type`: This is the type of call. It can be `call`, `storage` or `storage_proof`. <br/>

`parameters`: It takes additional parameter needed for call or storage call. In case of storage call or storage_proof call user can pass `block_number`.

#### ___`contract_address`___:
As name suggest first part of the method parameter is contract address.

#### ___`method_signature`___:
For contract call this is the method signature of the function we want to call. For storage call this is the position of storage we want to fetch value from.

#### __`call_type`__:
`call`: This is used to call a function in contract to get some value from it. Contract send calls are managed by universal wallet.

`storage`: This call type can be used to fetch storage from given contract provided the position of storage.

`storage_proof`: It returns proof of a value stored in contract storage at a given position.

`transaction_receipt`: This call type can be used to fetch transaction receipt of specified transaction.
