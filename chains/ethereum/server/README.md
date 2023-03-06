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


### `config`:
    This method returns `BlockchainConfig` which contains the configuration specific details for ethereum chain.

### `genesis_block`:
    Returns genesis block identifier.

### `node_version`:
    Returns node client version.

### `current_block`:
    Fetches current block using RPC and returns its identifier.

### `balance`:
    Fetches account balance from on chain and returns it. It takes two arguments:
    `address`: Address of account we want to fetch balance of.
    `block`: block identifier of block at which we want to fetch balance of account.

### `block`:
    This function takes `PartialBlockIdentifier` which contains a block index or hash and returns block transaction and operations happened in that transaction.

### `block_transaction`:
    This function takes: 
    `block`: Which is a block identifier of block from which we want to fetch transaction from.
    `tx`: Transaction identifier of transaction we want to fetch.
    And returns a specific transaction and its operations within specified block.

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

### `faucet`:
    This method is used to fund an account with some amount of tokens in testnet. It takes two arguments:
    `address`: Address of account we want to fund.
    `amount`: Amount of tokens we want to fund.

### `metadata`:
    This call is used to fetch nonce of account, It takes two arguments:
    `public_key`: This is the public key of sender.
    `options`: This is Params needed to create metadata. For ethereum chain it takes
        `destination`: Address of receivier.
        `amount`: Amount to be transfered to receiver.
        `data`: encoded input data for call

     It returns `EthereumMetadata` which includes `chain_id`, `nonce` and gas details for transaction.

### `submit`:
    It takes transaction bytes which is signed transaction bytes and it Submits signed transaction to chain and return its transaction id.
    
