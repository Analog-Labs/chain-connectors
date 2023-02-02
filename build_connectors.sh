# #!/bin/sh
TARGET_CC=x86_64-linux-musl-gcc cargo build -p rosetta-server-bitcoin --target x86_64-unknown-linux-musl --release
mkdir -p target/release/bitcoin/bin
cp target/x86_64-unknown-linux-musl/release/rosetta-server-bitcoin target/release/bitcoin/bin
docker build target/release/bitcoin -f chains/bitcoin/Dockerfile -t analoglabs/connector-bitcoin

TARGET_CC=x86_64-linux-musl-gcc cargo build -p rosetta-server-ethereum --target x86_64-unknown-linux-musl --release
mkdir -p target/release/ethereum/bin
cp target/x86_64-unknown-linux-musl/release/rosetta-server-ethereum target/release/ethereum/bin
docker build target/release/ethereum -f chains/ethereum/Dockerfile -t analoglabs/connector-ethereum

TARGET_CC=x86_64-linux-musl-gcc cargo build -p rosetta-server-polkadot --target x86_64-unknown-linux-musl --release
mkdir -p target/release/polkadot/bin
cp target/x86_64-unknown-linux-musl/release/rosetta-server-polkadot target/release/polkadot/bin
docker build target/release/polkadot -f chains/polkadot/Dockerfile -t analoglabs/connector-polkadot
