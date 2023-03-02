# __Rosetta Server for Substrate Chains__

This Project contains `BlockchainClient` impleemntation of ethereum chains.

Methods implemented are:
* `config`
* `genesis_block`
* `node_version`
* `current_block`
* `balance`
* `metadata`
* `submit`
* `block`
* `block_transaction`
* `call`


### `config`:
    This method returns `BlockchainConfig` which contains the configuration specific details for specific chain.

### `genesis_block`:
    Returns genesis block identifier.

### `node_version`:
    Returns node client version.

### `current_block`:
    Fetches current block using RPC and returns its identifier.

### `balance`:
    Fetches account balance from on chain and returns it. 

### `metadata`:
    This call is used to fetch nonce of account, and returns it with chain id and gas price for specified transaction.

### `submit`:
    Submit signed transaction to chain and returns the transaction id.

### `block`:
    This function takes a block index or hash and returns block transaction and operations happened in that transaction.

### `block_transaction`:
    This function returns a specific transaction and its operations within specified block.

### `call`:
    Required arguments:
    `method`: function signature of calling contract.
    `parameters`: Takes a mandatory param `type`
        `type` supports 3 paramters
        1. `call`
        2. `storage`
        3. `storage_proof`

        `call`: Takes a `contract_address` and returns parameter returned by function.

        `storage`: Takes `contract_address`, `position`, `block_number` and returns value   stored at position at specific `contract_address`.

        `storage_proof`: Takes `contract_address`, `position`, `block_number` and returns proof of value stored in that position at specific `contract_address`. It also does custom verification of proof and returns `true` or `false` based on verification.
    
