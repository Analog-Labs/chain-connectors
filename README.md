# Rosetta SDK

## Repository structure

- `rosetta-types`: initially autogenerated using the openapi-generator contains the request and
response structs used by the client and server.
- `rosetta-crypto`: cryptographic primitives used by the `rosetta-client`.
- `rosetta-client`: client to interact with a rosetta server.
- `rosetta-wallet`: command line interface built with the `rosetta-client`.
- `rosetta-cli`: command line interface built with the `rosetta-client`.
- `dioxus-wallet`: multichain mobile wallet
- `rosetta-server-substrate`: rosetta implementation for substrate chains
- `rosetta-server-polkadot`: rosetta implementation for polkadot using `rosetta-server-substrate`
- `rosetta-indexer`: generic block indexer for a rosetta connector

## Getting started

### Install cli tools
```
cargo install --path rosetta-cli
cargo install --path rosetta-wallet
```

### Bitcoin example
```
rosetta-wallet --chain btc --keyfile /tmp/alice faucet 1000
rosetta-wallet --chain btc --keyfile /tmp/bob account
rosetta-wallet --chain btc --keyfile /tmp/alice transfer ACCOUNT 1000
rosetta-wallet --chain btc --keyfile /tmp/alice faucet 1
rosetta-wallet --chain btc --keyfile /tmp/bob balance
```

### Ethereum example
```
rosetta-wallet --chain eth --keyfile /tmp/alice faucet 1000
rosetta-wallet --chain eth --keyfile /tmp/bob account
rosetta-wallet --chain eth --keyfile /tmp/alice transfer ACCOUNT 1000
rosetta-wallet --chain eth --keyfile /tmp/bob balance
```

### Substrate example
```
rosetta-wallet --chain dot --keyfile /tmp/alice faucet 3000000000000000
rosetta-wallet --chain dot --keyfile /tmp/bob account
rosetta-wallet --chain dot --keyfile /tmp/alice transfer bob_acc_key 1500000000000000
rosetta-wallet --chain dot --keyfile /tmp/bob balance
```

### Block Explorer
Open in your web browser [http://rosetta.analog.one:3000](http://rosetta.analog.one:3000)

### Run local testnet
Running a local testnet with `docker compose up` will start a bunch of containers:

- bitcoin: http://127.0.0.1:8080
- ethereum: http://127.0.0.1:8081
- polkadot: http://127.0.0.1:8082
- block explorer: [http://127.0.0.1:3000](http://127.0.0.1:3000)

Override the default url in `rosetta-cli` and `rosetta-wallet` with the `--url` flag.

## Update AWS deployment
Create a new tag, push to master and use it to create a new github release.
