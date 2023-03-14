This project is used to build transactions for ethereum chains.

## `EthereumTransactionBuilder`:
    Its implementation of `TransactionBuilder` and implements the following methods:
    1. `transfer`
    2. `method_call`
    3. `create_and_sign`

### `transfer`:
    Creates `EthereumMetadataParams` for transfer call.

### `method_call`:
    Creates `EthereumMetadataParams` for contract calls. It takes 
    `method`: this is a string contraining contract address and method signature with `-` seperation.
    `params`: array of json_value with params taken by contract methods in string format.

    It returns `EthereumMetadataParams` with `data` field set to the encoded contract call params.

### `create_and_sign`:
    When `metadata` is created we use this call to create Ethereum Transaction and sign it. It takes following arguments.
    `config`: chain sepecific config.
    `metadata_params`: Metadata params which created metadata for this call.
    `metadata`: Metadata required make transaction.
    `secret_key`: wallet's secret key (used to sign the transaction).

    It creates the transaction and signs it and then returns its bytes.


