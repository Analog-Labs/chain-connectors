#!/usr/bin/env bash
set -e

# Check for 'git' and abort if it is not available.
git --version > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'git' to get commit hash and tag."; exit 1; }

SOURCE_BASE_URL='https://github.com/Analog-Labs/chain-connectors'
REGISTRY_PATH='docker.io/analoglabs'
VCS_REF="$(git rev-parse HEAD)"
IMAGE_TAG='latest'

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

# Check if the rust target is installed
if ! rustup target list | grep -q "$rustTarget"; then
  echo "Installing the musl target with rustup '$rustTarget'"
  rustup target add "$rustTarget"
fi

# Detect host operating system
case $(uname -s) in
  # macOS
  Darwin)
    # Check if the musl linker is installed
    "$muslLinker" --version > /dev/null 2>&1 || { echo >&2 "ERROR - requires '$muslLinker' linker for compile"; exit 1; }

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

######################
## Helper Functions ##
######################

# Setup console colors
if test -t 1 && which tput >/dev/null 2>&1; then
    ncolors=$(tput colors)
    if test -n "$ncolors" && test "$ncolors" -ge 8; then
        bold_color=$(tput bold)
        green_color=$(tput setaf 2)
        warn_color=$(tput setaf 3)
        error_color=$(tput setaf 1)
        reset_color=$(tput sgr0)
    fi
    # 72 used instead of 80 since that's the default of pr
    ncols=$(tput cols)
fi
: "${ncols:=72}"

# Print the command in yellow, and the output in red
print_failed_command() {
  printf >&2 "$bold_color$warn_color%s$reset_color\n$error_color%s$reset_color\n" "$1" "$2"
}

# Execute a command and only print it if it fails
exec_cmd() {
  printf '  %s... ' "$1"
  local cmdOutput
  if eval "cmdOutput=\$( { $2 ;} 2>&1 )" > /dev/null; then
    # Success
    echo "$bold_color${green_color}OK$reset_color"
  else
    # Failure
    echo "$bold_color${error_color}FAILED$reset_color"
    print_failed_command "$2" "$cmdOutput"
    exit 1
  fi
}

# Check if the docker image contains the expected label and value
check_label() {
  local imageCID
  local label
  local value
  imageCID="$1"
  label="$2"
  value="$3"

  local cmd
  cmd="docker inspect -f '{{ index .ContainerConfig.Labels \"$label\" }}' $imageCID"
  cmd="[[ \"\$($cmd)\" == \"$value\" ]] || { echo \"wrong value: \$($cmd)\"; exit 1; }"
  exec_cmd "    - label '$label'" "$cmd"
}

# Build connector docker images and delete intermediary images
build_image() {
  local blockchain
  local imageTag
  local oldImageId
  local newImageId
  local buildDate
  local buildId
  local cmd

  blockchain="$1"
  imageTag="analoglabs/connector-$blockchain:$IMAGE_TAG"

  # Docker doesn't remove the old image automatically,
  # So we manually remove it later if the build succeed
  # Reference: https://stackoverflow.com/a/52477584
  oldImageId="$(docker images -qa --no-trunc "$imageTag" 2> /dev/null)"

  # Workaround for removing multi-stage intermediary images
  # Reference: https://stackoverflow.com/a/55082473
  buildId="$(hexdump -vn16 -e'4/4 "%08X" 1 "\n"' /dev/urandom)"
  buildDate="$(date +%Y%m%d)"
  printf -- '- %s\n' "$blockchain"

  # Build connector image
  cmd=(
    docker build "target/release/$blockchain"
       -f "chains/$blockchain/Dockerfile"
       -t "$imageTag"
       --build-arg "BUILD_ID=$buildId"
       --build-arg "REGISTRY_PATH=$REGISTRY_PATH"
       --build-arg "VCS_REF=$VCS_REF"
       --build-arg "BUILD_DATE=$buildDate"
       --build-arg "IMAGE_VERSION=$IMAGE_TAG"
       --force-rm
       --no-cache
  )
  exec_cmd "[1] building $imageTag image" "${cmd[*]}"

  # Delete intermediary multi-stage images
  # Ref: https://stackoverflow.com/a/55082473
  cmd=(
    docker image prune -f \
        --filter 'label=stage=certs' \
        --filter "label=build=$buildId"
  )
  exec_cmd '[2] deleting intermediary images' "${cmd[*]}"

  # Check image labels
  printf '  [3] checking '%s' labels...\n' "$imageTag"
  newImageId="$(docker images -qa "$imageTag" 2> /dev/null)"
  check_label "$newImageId" 'name' "$REGISTRY_PATH/connector-$blockchain"
  check_label "$newImageId" 'version' "$IMAGE_TAG"
  check_label "$newImageId" 'one.analog.image.created' "$buildDate"
  check_label "$newImageId" 'one.analog.image.revision' "$VCS_REF"
  check_label "$newImageId" 'one.analog.image.source' "$SOURCE_BASE_URL/blob/$VCS_REF/chains/$blockchain/Dockerfile"
  check_label "$newImageId" 'one.analog.image.vendor' "Analog One Foundation"

  # Delete old image if it is dangling
  if [[ "$(docker image inspect -f '{{ .RepoTags }}' "$oldImageId" 2> /dev/null)" == "[]" ]]; then
    cmd=(docker rmi "$oldImageId")
    exec_cmd '[4] deleting old dangling image' "${cmd[*]}"
  fi
  printf '\n'
}

# Build Bitcoin Connector
build_image 'bitcoin'

# Build Ethereum Connector
build_image 'ethereum'

# Build Polkadot Connector
build_image 'polkadot'

# Build Astar Connector
build_image 'astar'
