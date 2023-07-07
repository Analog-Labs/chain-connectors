#!/usr/bin/env bash
set -e

REPO=https://github.com/Analog-Labs/chain-connectors
REGISTRY_PATH=${REGISTRY_PATH:-analoglabs}
DOCKER_IMAGE_NAME=base-ci-linux
DOCKER_IMAGE_VERSION=latest
CONNECTOR_IMAGE_VERSION=0.4.0

# Check if docker is running
if ! docker info > /dev/null 2>&1; then
  echo "This script uses docker - please start docker and try again!"
  exit 1
fi

# Build the base-ci-linux if necessary
if [[ "$(docker images -q "${REGISTRY_PATH}/${DOCKER_IMAGE_NAME}:${DOCKER_IMAGE_VERSION}" 2> /dev/null)" == "" ]]; then
  docker build \
    -f ./ci/dockerfiles/base-ci-linux/Dockerfile \
    --build-arg VCS_REF=$(git rev-parse HEAD) \
    --build-arg BUILD_DATE=$(date +%Y%m%d) \
    --no-cache \
    -t "${REGISTRY_PATH}/${DOCKER_IMAGE_NAME}:${DOCKER_IMAGE_VERSION}" \
    "./ci/dockerfiles/${DOCKER_IMAGE_NAME}"
fi

docker build \
    -f ./ci/dockerfiles/builder/Dockerfile \
    --no-cache \
    -t "${REGISTRY_PATH}/builder:latest" \
    .

docker build \
  -f ./chains/bitcoin/Dockerfile \
  -t analoglabs/connector-bitcoin:${CONNECTOR_IMAGE_VERSION} \
  ./chains/bitcoin

docker build \
  -f ./chains/ethereum/Dockerfile \
  -t analoglabs/connector-ethereum:${CONNECTOR_IMAGE_VERSION} \
  ./chains/ethereum

docker build \
  -f ./chains/polkadot/Dockerfile \
  -t analoglabs/connector-polkadot:${CONNECTOR_IMAGE_VERSION} \
  ./chains/polkadot

docker build \
  -f ./chains/astar/Dockerfile \
  -t analoglabs/connector-astar:${CONNECTOR_IMAGE_VERSION} \
  ./chains/astar

exit 0
cargo build -p rosetta-server-bitcoin --target x86_64-unknown-linux-musl --release
mkdir -p target/release/bitcoin/bin
cp target/x86_64-unknown-linux-musl/release/rosetta-server-bitcoin target/release/bitcoin/bin
docker build target/release/bitcoin -f chains/bitcoin/Dockerfile -t analoglabs/connector-bitcoin

cargo build -p rosetta-server-ethereum --target x86_64-unknown-linux-musl --release
mkdir -p target/release/ethereum/bin
cp target/x86_64-unknown-linux-musl/release/rosetta-server-ethereum target/release/ethereum/bin
docker build target/release/ethereum -f chains/ethereum/Dockerfile -t analoglabs/connector-ethereum

cargo build -p rosetta-server-polkadot --target x86_64-unknown-linux-musl --release
mkdir -p target/release/polkadot/bin
cp target/x86_64-unknown-linux-musl/release/rosetta-server-polkadot target/release/polkadot/bin
docker build target/release/polkadot -f chains/polkadot/Dockerfile -t analoglabs/connector-polkadot

cargo build -p rosetta-server-astar --target x86_64-unknown-linux-musl --release
mkdir -p target/release/astar/bin
cp target/x86_64-unknown-linux-musl/release/rosetta-server-astar target/release/astar/bin
docker build target/release/astar -f chains/astar/Dockerfile -t analoglabs/connector-astar
