This project is used to build transactions for substrate chains.

## `PolkadotTransactionBuilder`:
    Its implementation of `TransactionBuilder` and implements the following methods:
    1. `transfer`
    2. `method_call`
    3. `create_and_sign`

### `transfer`:
    Creates `PolkadotMetadataParams` for transfer call.

### `method_call`:
    Not implemented.

### `create_and_sign`:
    When `metadata` is created we use this call to create Ethereum Transaction and sign it. It takes following arguments.
    `config`: chain sepecific config.
    `metadata_params`: Metadata params which created metadata for this call.
    `metadata`: Metadata required make transaction.
    `secret_key`: wallet's secret key (used to sign the transaction).

    It creates the transaction and signs it and then returns it in bytes.
