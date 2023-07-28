#!/usr/bin/env bash
set -e

# Check for 'git' and abort if it is not available.
git --version > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'git' to get commit hash and tag."; exit 1; }

REGISTRY_PATH="docker.io/analoglabs"
VCS_REF="$(git rev-parse HEAD)"
IMAGE_TAG="latest"

# Check for 'uname' and abort if it is not available.
uname -v > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'uname' to identify the platform."; exit 1; }

# Check for 'docker' and abort if it is not running.
docker info > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'docker', please start docker and try again."; exit 1; }

# Check for 'rustup' and abort if it is not available.
rustup -V > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'rustup' for compile the binaries"; exit 1; }

# Detect host architecture
case "$(uname -m)" in
    x86_64)
        rustTarget='x86_64-unknown-linux-musl'
        muslLinker='x86_64-linux-musl-gcc'
        ;;
    arm64|aarch64)
        rustTarget='aarch64-unknown-linux-musl'
        muslLinker='aarch64-linux-musl-gcc'
        ;;
    *)
        echo >&2 "ERROR - unsupported architecture: $(uname -m)"
        exit 1
        ;;
esac

# Check if the musl linker is installed
"$muslLinker" --version > /dev/null 2>&1 || { echo >&2 "ERROR - requires '$muslLinker' linker for compile"; exit 1; }

# Check if the rust target is installed
if ! rustup target list | grep -q "$rustTarget"; then
  echo "Installing the musl target with rustup '$rustTarget'"
  rustup target add "$rustTarget"
fi

# Detect host operating system
case $(uname -s) in
  # macOS
  Darwin)
    buildArgs=(
      --release
      --target "$rustTarget"
      --config "target.$rustTarget.linker='$muslLinker'"
      --config "env.CC_$rustTarget='$muslLinker'"
    )
    ;;
  # Linux
  Linux)
    buildArgs=(
      --release
      --target "$rustTarget"
    )
    ;;
  *)
    echo >&2 "ERROR - unsupported or unidentified operating system: $(uname -s)"
    exit 1
    ;;
esac

# Build all Connectors
cargo build \
  -p rosetta-server-bitcoin \
  -p rosetta-server-polkadot \
  -p rosetta-server-ethereum \
  -p rosetta-server-astar \
  "${buildArgs[@]}" || exit 1

# Move binaries
mkdir -p target/release/{bitcoin,ethereum,polkadot,astar}/bin
cp "target/$rustTarget/release/rosetta-server-bitcoin" target/release/bitcoin/bin
cp "target/$rustTarget/release/rosetta-server-ethereum" target/release/ethereum/bin
cp "target/$rustTarget/release/rosetta-server-polkadot" target/release/polkadot/bin
cp "target/$rustTarget/release/rosetta-server-astar" target/release/astar/bin

# Build Bitcoin Connector
docker build target/release/bitcoin \
  -f chains/bitcoin/Dockerfile \
  -t analoglabs/connector-bitcoin \
  --build-arg "REGISTRY_PATH=$REGISTRY_PATH" \
  --build-arg "VCS_REF=$VCS_REF" \
  --build-arg "BUILD_DATE=$(date +%Y%m%d)" \
  --build-arg "IMAGE_VERSION=$IMAGE_TAG" \
  --no-cache

# Build Ethereum Connector
docker build target/release/ethereum \
  -f chains/ethereum/Dockerfile \
  -t analoglabs/connector-ethereum \
  --build-arg "REGISTRY_PATH=$REGISTRY_PATH" \
  --build-arg "VCS_REF=$VCS_REF" \
  --build-arg "BUILD_DATE=$(date +%Y%m%d)" \
  --build-arg "IMAGE_VERSION=$IMAGE_TAG" \
  --no-cache

# Build Polkadot Connector
docker build target/release/polkadot \
  -f chains/polkadot/Dockerfile \
  -t analoglabs/connector-polkadot \
  --build-arg "REGISTRY_PATH=$REGISTRY_PATH" \
  --build-arg "VCS_REF=$VCS_REF" \
  --build-arg "BUILD_DATE=$(date +%Y%m%d)" \
  --build-arg "IMAGE_VERSION=$IMAGE_TAG" \
  --no-cache

# Build Astar Connector
docker build target/release/astar \
  -f chains/astar/Dockerfile \
  -t analoglabs/connector-astar \
  --build-arg "REGISTRY_PATH=$REGISTRY_PATH" \
  --build-arg "VCS_REF=$VCS_REF" \
  --build-arg "BUILD_DATE=$(date +%Y%m%d)" \
  --build-arg "IMAGE_VERSION=$IMAGE_TAG" \
  --no-cache
