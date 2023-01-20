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
    "method": "pallet_name, storage_name, query_type",
    "params": [
        ["A", ""], //representing TestEnum::A
        [1, 2, 3] //representing TestStruct { a: 1, b: 2, c: 3 }
    ]
}
```

Other primitive types are passed as they are. e.g.
```Json
{
    "method": "pallet_name, storage_name, query_type",
    "params": [
        1,          //representing u32
        "test",     //representing String
        false,      //representing bool
    ]
}
```





