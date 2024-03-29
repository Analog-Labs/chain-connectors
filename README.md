# Rosetta SDK Documentation

The Rosetta is a set of tools for blockchain integration, it's goal is make blockchain integration simpler, faster, and more reliable. This repo provides a framework for Analog’s chronicles — special nodes hosted by time node operators — to simplify their interactions with Analog-connected chains in a manner compliant with the Analog Network’s protocol.

## Repository structure

This repo contains the following modules:

- `rosetta-core`. Provides traits and definitions shared by the server and client crates.
- `rosetta-server`. This is a generic implementation of the Rosetta Server. The Rosetta Server is a standalone server that a connector on any Analog-supported chain can connect to and listen to the port specified in the settings.
- `rosetta-client`. This is a standard client that interacts with the Rosetta Server.
- `rosetta-types`. It contains the request and response structs used by the client and server. It is initially autogenerated using the openapi-generator.
- `rosetta-crypto`. It has cryptographic primitives used by the rosetta-client.
- `rosetta-wallet`. This is a command line interface (CLI) built with the rosetta-client.
- `rosetta-cli`. This is a CLI built with the rosetta-client.
- `rosetta-docker`. This is a generic Rosetta Server testing infrastructure.
- `chains`. These are chain-specific client/server components.

## Getting started

<!--This section needs to be refined -->

To get started with the Rosetta SDK, ensure you have following dependencies installed:

- [rust](https://www.rust-lang.org/)
- [latest version of Docker](https://www.docker.com/get-started/)
- solc 0.8.20+, recommend install using svm: https://github.com/alloy-rs/svm-rs
- [dprint](https://github.com/dprint/dprint): `cargo install --locked dprint`
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny): `cargo install --locked cargo-deny`

Build

```shell
$ cargo build -p rosetta-client
```

Lint

```shell
$ cargo +nightly fmt --all -- --check
$ cargo clippy --locked --workspace --examples --tests --all-features -- \
  -Dwarnings \
  -Dclippy::unwrap_used \
  -Dclippy::expect_used \
  -Dclippy::nursery \
  -Dclippy::pedantic \
  -Aclippy::module_name_repetitions
$ dprint check
$ cargo deny check
```

Run unit tests

```shell
$ cargo test --workspace --all-features \
  --exclude rosetta-testing-arbitrum \
  --exclude rosetta-server-astar \
  --exclude rosetta-server-ethereum \
  --exclude rosetta-server-polkadot \
  --exclude rosetta-client
```

Run integration tests

```shell
# Pull docker images
./scripts/pull_nodes.sh

# Run tests
$ cargo test \
  -p rosetta-server-astar \
  -p rosetta-server-ethereum \
  -p rosetta-server-polkadot \
  -p rosetta-client
```

Run arbitrum integration tests

```shell
# Setup arbitrum local testnet
git clone -b release --depth=1 --no-tags --recurse-submodules https://github.com/ManojJiSharma/nitro-testnode.git
cd nitro-testnode
./test-node.bash --detach
cd ..

# Run tests
cargo test --locked -p rosetta-testing-arbitrum
```

## Contributing

You can contribute to this repo in a number of ways, including:

- [Asking questions](https://github.com/Analog-Labs/chain-connectors/issues/new?assignees=&labels=question&template=ask-a-question.md&title=)
- [Giving feedback](https://github.com/Analog-Labs/chain-connectors/issues/new?assignees=&labels=enhancement&template=suggest-a-feature.md&title=)
- [Reporting bugs](https://github.com/Analog-Labs/chain-connectors/issues/new?assignees=&labels=bug&template=report-a-bug.md&title=)
  Read our [contribution guidelines](https://github.com/Analog-Labs/.github-private/wiki/Contribution-Guidelines) for more information on how to contribute to this repo.
