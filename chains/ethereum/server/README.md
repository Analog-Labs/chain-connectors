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
### `block_transaction`:
### `call`:
