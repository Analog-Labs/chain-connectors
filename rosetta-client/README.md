This get starter to run example available in rosetta-client.

## Ethereum Setting up nodes

### MacOS

if you're on macos, install [musl-cross](https://github.com/FiloSottile/homebrew-musl-cross) for enable musl-target cross-compilation:

```shell
brew install filosottile/musl-cross/musl-cross --with-aarch64
```

### Debian/Ubuntu

if you're on debian/ubuntu, install [musl-tools](https://packages.debian.org/sid/musl-tools) for enable musl-target cross-compilation:

```shell
apt-get install musl-tools
```

### Shared Steps

1. First you need to build connectors for that you can run `./build_connectors.sh`
2. After connectors are build run `docker compose up -d`.

### Compiling voting contract

1. We have a `voting_contract.sol` we have to compile and get its binary in order to deploy it. For this you need to have `solc` installed. You can install it using `brew install solidity` or `sudo apt-get install solc`.
2. Run `solc --combined-json abi,bin --abi --bin voting_contract.sol -o ./voting_contract_files` in contract folder.
3. You will get `voting_contract_files` folder with `voting_contract.abi`, `voting_contract.bin` and `combined_voting_contract.json` which contains both abi and bin since we are only concerned with bin we will use `voting_contract.bin`. and for sake of easiness we have already compiled and imported it in examples folder.

### Running voting_contract example

1. This example demonstrate how to interact with smart contract using Analog's wallet. We will deploy a basic contracts storing yes or no votes and displays total votes on voting.
2. Run `cargo run --example voting_contract faucet`. to get some funds to deploy contract.
3. To deploy contract run `cargo run --example voting_contract deploy`. You will get deployed contract address as output, make sure you copy it.
4. To vote for yes run
   `cargo run --example voting_contract vote --contract-address "0x678ea0447843f69805146c521afcbcc07d6e28a2" -v`
   To vote for no run
   `cargo run --example voting_contract vote --contract-address "0x678ea0447843f69805146c521afcbcc07d6e28a2"`
   you will get `CallResponse` as output containing n array first uint is total of `yes` votes and second for `no` votes in contract.

### Running ethereum example

1. This examples demonstrate how to interact with ethereum using Analog's wallet.
2. Make sure you have voting contract deployed. If not please follow voting_contract example steps 2 and 3.
3. Run `cargo run --example ethereum -- --contract-address "0x678ea0447843f69805146c521afcbcc07d6e28a2"`
4. It runs all available methods available for wallet and respond with valid output.
