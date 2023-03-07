# __Rosetta Server for Substrate Chains__

This Project contains `BlockchainClient` implementation of substrate chains.

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

This method returns `BlockchainConfig` which contains the configuration specific details for polkadot chain.

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


### `call`:

To fetch Storage or any Constant from a Substrate chain, you can use the `Call` function. This method takes a `CallRequest` as input and returns a json `Value` as output. The `CallRequest` contains the following fields:
* `NetworkIdentifier` - The network to make the call on.
* `Method` - A string contains `-` seperated the name of pallet, function of pallet and type of query e.g. `Storage`, `Constant`. 
* `Parameters` - A Array containing the parameters required for that pallet function.

### __Passing paramters__
`Enums` and `Struct` are passed in paramter as array values
lets say we have an enum:
```Rust
enum TestEnum {
    A,
    B(u32),
    C,
}
```
and we want to pass `TestEnum::A` as parameter then we pass `["A", ""]` as parameter.
Second value in array is for the paramter of enum variant. In this case its will be empty since enum does not contain any paramter.
if we want to pass `TestEnum::B(10)` as parameter then we pass `["B", 10]` as parameter. 
Here we passed 10 as a Json Number in array.

Now lets take a look if a function call requires a some complex enum like:
```Rust
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
pub enum MultiAddress<AccountId, AccountIndex> {
    /// It's an account ID (pubkey).
    Id(AccountId),
    /// It's an account index.
    Index(#[codec(compact)] AccountIndex),
    /// It's some arbitrary raw bytes.
    Raw(Vec<u8>),
    /// It's a 32 byte representation.
    Address32([u8; 32]),
    /// Its a 20 byte representation.
    Address20([u8; 20]),
}
```
we need to pass Id variant of this enum now this variant takes a parameter which is `AccountId` type. Type of `AccountId` is 

```Rust
type AccountId = AccountId32;
```
where `AccountId32` is a struct with 32 byte vector.

```Rust
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
pub struct AccountId32(pub [u8; 32]);
```

So we need to pass `AccountId32` as a parameter. We can pass it as `[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]` which is a 32 byte array. and complete enum param will look like this

```Json
{
    "params": [
        ["Id", [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]]
    ]
}

```

Similarly if we want to pass a `Struct` in parameters then we pass it as array of values as well.
let's say we have a struct:

```Rust
struct TestStruct {
    a: u32,
    b: u32,
    c: u32,
}
```
and we want to pass `TestStruct { a: 1, b: 2, c: 3 }` as parameter then we pass `[1, 2, 3]` as parameter array.
So passing those both as a paramter list should look like this
```Json
{
    "method": "pallet_name-storage_name-query_type",
    "params": [
        ["A", ""], //representing TestEnum::A
        [1, 2, 3] //representing TestStruct { a: 1, b: 2, c: 3 }
    ]
}
```

Other primitive types are passed as they are. e.g.
```Json
{
    "method": "pallet_name-storage_name-query_type",
    "params": [
        1,          //representing u32
        "test",     //representing String
        false,      //representing bool
    ]
}
```
