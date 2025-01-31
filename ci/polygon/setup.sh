#!/usr/bin/env bash
#set -e

###################
## Configuration ##
###################
# default parameters
BaseDirectory="${1-$PWD}"
LogFile="$BaseDirectory/config.log"
CodeDirectory="$BaseDirectory/code"
BorDirectory="$CodeDirectory/bor"
HeimdallDirectory="$CodeDirectory/heimdall"
BackupFile="$CodeDirectory/polygon-config.tar.gz"

NumOfBorValidators='3'
NumOfBorArchiveNodes='0'
NumOfBorSentries='0'

NumOfErigonValidators='0'
NumOfErigonSentries='0'
NumOfErigonArchiveNodes='0'
HeimdallChainId='heimdall-15001'
BorChainId="15001"

genesisContractsBranch='master'
contractsBranch='mardizzone/node-16'
defaultStake='10000'
blockNumber=('0')
blockTime=('2')
sprintSize=('64')
sprintSizeBlockNumber=('0')

#######################
### HELPER FUNCTIONS ##
#######################
# Setup console colors
if test -t 1 && which tput >/dev/null 2>&1; then
    ncolors=$(tput colors)
    if test -n "$ncolors" && test $ncolors -ge 8; then
        bold_color=$(tput bold)
        warn_color=$(tput setaf 3)
        error_color=$(tput setaf 1)
        reset_color=$(tput sgr0)
    fi
    # 72 used instead of 80 since that's the default of pr
    ncols=$(tput cols)
fi
: ${ncols:=72}

log(){
    printf '%s\n' "$@" >> "$LogFile"
}

log_cmd(){
  log "$(printf '%s $ %s' "$(date '+%Y-%m-%d %H:%M:%S')" "$1")"
}

warn(){
  log "WARNING: $*"
  echo "$warn_color$bold_color$*$reset_color"
}

die(){
  log "ERROR: $*"
  echo "$error_color$bold_color$*$reset_color"
  exit 1
}

failed_command(){
  echo "$error_color${bold_color}Failed executing command:$reset_color $warn_color$1$reset_color"
  echo "check the config.log for more information"
  exit 1
}

exec_cmd() {
    log_cmd "$1"
    eval "{ $1 ;} > >(tee -a '$LogFile') 2> >(tee -a '$LogFile')" &> /dev/null || failed_command "$1";
    log ""
}

file_replace() {
    local regex="$1"
    local replacement="$2"
    local inputFile="$3"
    local outputFile="$4"

    file_exists "$inputFile"
    if [[ "$outputFile" == '' ]]; then
      if [[ ! -f "$inputFile.backup" ]]; then
        exec_cmd "mv '$inputFile' '$inputFile.backup'"
      fi
      outputFile="$inputFile"
      inputFile="$inputFile.backup"
    fi

    exec_cmd "sed -re 's/$regex/$replacement/gi' '$inputFile' > '$outputFile'"
}

test_cmd(){
    log "$@"
    "$@" >> "$LogFile" 2>&1
}

file_exists() {
  if [[ ! -f "$1" ]]; then
      die "File not found: \"$1\""
  fi
}

directory_exists() {
  if [[ ! -d "$1" ]]; then
      die "Directory not found: \"$1\""
  fi
}

# Using a pwd+dirname instead of realpath because of an issue on macos
# http://biercoff.com/readlink-f-unrecognized-option-problem-solution-on-mac/
resolve_path() {
  local absolutePath
  if [[ -f "$1" ]]; then
      local fileName
      fileName=$(basename -- "$1")
      absolutePath=$(dirname -- "$1") # relative
      absolutePath=$(cd -- "$absolutePath" && pwd) # absolutized and normalized
      absolutePath="$absolutePath/$fileName"       # fully qualified path
  elif [[ -d "$1" ]]; then
    absolutePath=$(cd -- "$1" && pwd) # absolutized and normalized
  else
      die "File or directory not found: \"$1\""
  fi
  echo "$absolutePath"
}

check_repository() {
  local directoryPath
  local expectedGitUrl

  directoryPath=$(resolve_path "$1")
  expectedGitUrl="$2"

  # Check if the directory exists
  if [[ ! -d "$directoryPath" ]]; then
      die "Directory not found: \"$1\""
  fi

  local gitUrl
  gitUrl=$(cd -- "$directoryPath" && git config --get remote.origin.url)

  # Check if the directory is a git repository
  if [[ "$gitUrl" != "$expectedGitUrl" ]]; then
      echo "The expect git repository doesn't match."
      warn " expect: \"$expectedGitUrl\""
      warn " actual: \"$gitUrl\""
      exit 1
  fi
}

##########################
### CHECK DEPENDENCIES ###
##########################
if ! command -v go &> /dev/null; then
    die "Golang not found, install it from https://golang.org/dl/"
fi

if ! command -v node &> /dev/null; then
    die "Nodejs not found, install it from https://nodejs.org/en"
fi

if ! command -v solc &> /dev/null; then
    die "Solc v0.5.17 not found"
fi

if ! command -v jq &> /dev/null; then
    echo "$error_color${bold_color}jq not found.$reset_color"
    echo " For MacOs: brew install jq"
    echo "For Debian: apt-get install jq"
    exit 1
fi

# Check required config directories exists
directory_exists "$BaseDirectory/config"
directory_exists "$BaseDirectory/data"
directory_exists "$BaseDirectory/devnet"

exec_cmd "cd '$BaseDirectory'"

# Create code directory if it doesn't exist
if [[ ! -d "$CodeDirectory" ]]; then
  exec_cmd "mkdir -p '$CodeDirectory'"
fi

# Checkout Bor and Heimdall code
if [[ ! -d "$BorDirectory" ]]; then
  exec_cmd "mkdir -p '$BorDirectory'"
  exec_cmd "git clone 'https://github.com/maticnetwork/bor.git' --branch 'v0.4.0' --depth 1 '$BorDirectory'"
fi

if [[ ! -d "$HeimdallDirectory" ]]; then
  exec_cmd "mkdir -p '$HeimdallDirectory'"
  exec_cmd "git clone 'https://github.com/maticnetwork/heimdall.git' --branch 'v0.3.4' --depth 1 '$HeimdallDirectory'"
fi

# Check if the repository folders are valid
check_repository "$BorDirectory" 'https://github.com/maticnetwork/bor.git'
check_repository "$HeimdallDirectory" 'https://github.com/maticnetwork/heimdall.git'

# Convert relative paths to absolute paths
BorDirectory=$(resolve_path "$BorDirectory")
HeimdallDirectory=$(resolve_path "$HeimdallDirectory")

# Check if the binaries are built
if [[ "$BaseDirectory" == "$BorDirectory" ]]; then
    die "Cannot build from bor repository directory"
fi
if [[ "$BaseDirectory" == "$HeimdallDirectory" ]]; then
    die "Cannot build from heimdall repository directory"
fi

# Cleanup previous execution
if [[ -f "$LogFile" ]]; then
  rm "$LogFile"
fi

# If the backup file exists, restore the config files
if [[ -f "$BackupFile" ]]; then
  exec_cmd 'rm -rf config data devnet'
  exec_cmd "tar -xzvf '$BackupFile'"
fi

# If the backup doesn't exists, create it
if [[ ! -f "$BaseDirectory/polygon-config.tar.gz" ]]; then
  exec_cmd "tar -czvf '$BaseDirectory/polygon-config.tar.gz' config data devnet"
fi

#############################
### SETUP POLYGON TESTNET ###
#############################
build_heimdall() {
  if [[ -f "$HeimdallDirectory/build/heimdalld" ]] && [[ -f "$HeimdallDirectory/build/heimdallcli" ]]; then
    echo "Skipping Heimdall build..."
    return
  fi

  echo "Building Heimdall..."
  exec_cmd "cd '$HeimdallDirectory'"
  [[ -d 'build' ]] && exec_cmd 'rm -rf build'
  exec_cmd 'mkdir -p build' || die "cannot create build directory at \"$HeimdallDirectory/build\""

  # Get the tag version, ex: 0.3.4
  local VERSION
  VERSION=$(git --no-pager describe --tags | sed 's/^v//')

  # Get the commit hash
  local COMMIT
  COMMIT=$(git --no-pager log -1 --format='%H')

  # Build flags
  local BUILD_FLAGS
  BUILD_FLAGS="-X github.com/maticnetwork/heimdall/version.Name=heimdall -X github.com/maticnetwork/heimdall/version.ServerName=heimdalld -X github.com/maticnetwork/heimdall/version.ClientName=heimdallcli -X github.com/maticnetwork/heimdall/version.Version=$VERSION -X github.com/maticnetwork/heimdall/version.Commit=$COMMIT -X github.com/cosmos/cosmos-sdk/version.Name=heimdall -X github.com/cosmos/cosmos-sdk/version.ServerName=heimdalld -X github.com/cosmos/cosmos-sdk/version.ClientName=heimdallcli -X github.com/cosmos/cosmos-sdk/version.Version=$VERSION -X github.com/cosmos/cosmos-sdk/version.Commit=$COMMIT"

  # Build heimdalld and heimdallcli
  exec_cmd "go build -ldflags '$BUILD_FLAGS' -o build/heimdalld ./cmd/heimdalld"
  exec_cmd "go build -ldflags '$BUILD_FLAGS' -o build/heimdallcli ./cmd/heimdallcli"

  # Check if the binaries exists
  file_exists './build/heimdalld'
  file_exists './build/heimdallcli'

  # Print version
  exec_cmd './build/heimdalld version'
  exec_cmd './build/heimdallcli version'

  echo "Heimdall built successfully!!"
}

build_bor() {
  echo "Building Bor..."
  exec_cmd "cd '$BorDirectory'"
  exec_cmd 'make bor'
  echo "Bor built successfully!!"
}

create_heimdall_testnet_files() {
  echo "Create testnet files for Heimdall"
  exec_cmd "cd '$BaseDirectory'"
  # Number of validators to initialize the testnet with (default 4)
  local validatorCount=$((NumOfBorValidators + NumOfErigonValidators))

  # Number of non-validators to initialize the testnet with (default 8)
  local nonValidatorCount="$((NumOfBorSentries + NumOfBorArchiveNodes + NumOfErigonSentries + NumOfErigonArchiveNodes))"

  local totalBorNodes=$((NumOfBorValidators + NumOfBorSentries + NumOfBorArchiveNodes))
  local totalErigonNodes=$((NumOfErigonValidators + NumOfErigonSentries + NumOfErigonArchiveNodes))
  local totalNodes=$((totalBorNodes + totalErigonNodes))

  local HEIMDALL_CMD
  HEIMDALL_CMD="$HeimdallDirectory/build/heimdalld"

  # Create testnet files
  exec_cmd "$HEIMDALL_CMD create-testnet \
--home devnet \
--v "$validatorCount" \
--n "$nonValidatorCount" \
--chain-id "$HeimdallChainId" \
--node-host-prefix heimdall \
--output-dir devnet"

  # set heimdall peers with devnet heimdall hosts
  for (( node=0; node < totalNodes; node++ ))
  do
    local heimdallConfigFilePath
    local heimdallGenesisFilePath
    heimdallConfigFilePath="$BaseDirectory/devnet/node$node/heimdalld/config/config.toml"
    heimdallGenesisFilePath="$BaseDirectory/devnet/node$node/heimdalld/config/genesis.json"

    file_replace '^moniker[[:blank:]]=.*$' \
      "moniker = \"heimdall$node\"" \
      "$heimdallConfigFilePath"

    file_replace '"bor_chain_id"[ ]*:[ ]*"[^"]*"' \
      "\"bor_chain_id\": \"$BorChainId\"" \
      "$heimdallGenesisFilePath"
  done
}

setup_genesis_contracts() {
  echo "Setup genesis contracts"
  local defaultBalance
  defaultBalance='1000000000' # 1 Billion - Without 10^18

  directory_exists "$CodeDirectory"

  if [[ ! -d "$CodeDirectory/genesis-contracts" ]]; then
    echo "Cloning genesis-contracts repository"
    exec_cmd "cd '$CodeDirectory'"
    exec_cmd "git clone 'https://github.com/maticnetwork/genesis-contracts' --branch '$genesisContractsBranch' genesis-contracts"
    exec_cmd "cd '$CodeDirectory/genesis-contracts'"
    exec_cmd 'npm install --omit=dev'
    exec_cmd 'git submodule init && git submodule update'
  else
    check_repository "$CodeDirectory/genesis-contracts" 'https://github.com/maticnetwork/genesis-contracts'
    echo "Updating genesis-contracts repository"
    exec_cmd "cd '$CodeDirectory/genesis-contracts'"
    exec_cmd "git checkout '$genesisContractsBranch'"
    exec_cmd 'npm install --omit=dev'
    exec_cmd 'git submodule update'
  fi

  echo "Install dependencies for matic-contracts"
  directory_exists "$CodeDirectory/genesis-contracts/matic-contracts"
  exec_cmd "cd '$CodeDirectory/genesis-contracts/matic-contracts'"
  exec_cmd "npm install --omit=dev"

  echo "Process templates"
  exec_cmd "npm run template:process -- --bor-chain-id '$BorChainId'"

  echo "Compile matic-contracts"
  exec_cmd 'npm run truffle:compile'

  echo "Prepare validators for genesis file"
  local signerDumpFile="$BaseDirectory/devnet/signer-dump.json"
  local validatorsFile="$CodeDirectory/genesis-contracts/validators.json"
  local jqFilter=". |= map({ \"address\": .address, \"stake\": $defaultStake, \"balance\": $defaultBalance })"
  exec_cmd "jq '$jqFilter' '$signerDumpFile' > '$validatorsFile'"
  if [[ -f "$CodeDirectory/genesis-contracts/validators.js" ]]; then
    exec_cmd "mv '$CodeDirectory/genesis-contracts/validators.js' '$CodeDirectory/genesis-contracts/validators.js.backup'"
  fi

  echo "Configure Block time"
  local blockFile="$CodeDirectory/genesis-contracts/blocks.json"
  local blocksJson
  blocksJson="["
  for (( block=0; block < "${#blockNumber[@]}"; block++ )); do
    blocksJson="$blocksJson{\"number\": \"${blockNumber["$block"]}\", \"time\": \"${blockTime["$block"]}\"}"
  done
  blocksJson="$blocksJson]"
  exec_cmd "printf '%s' \"\$(echo '$blocksJson' | jq '.')\" > '$blockFile'"
  if [[ -f "$CodeDirectory/genesis-contracts/blocks.js" ]]; then
    exec_cmd "mv '$CodeDirectory/genesis-contracts/blocks.js' '$CodeDirectory/genesis-contracts/blocks.js.backup'"
  fi

  echo "Configure Sprint Size"
  local sprintSizesFile="$CodeDirectory/genesis-contracts/sprintSizes.json"
  local sprintSizesJson
  sprintSizesJson="["
  for (( block=0; block < "${#sprintSize[@]}"; block++ )); do
    sprintSizesJson="$sprintSizesJson{\"number\": \"${sprintSizeBlockNumber["$block"]}\", \"sprintSize\": \"${sprintSize["$block"]}\"}"
  done
  sprintSizesJson="$sprintSizesJson]"

  # Save file
  exec_cmd "printf '%s' \"\$(echo '$sprintSizesJson' | jq '.')\" > '$sprintSizesFile'"
  if [[ -f "$CodeDirectory/genesis-contracts/sprintSizes.js" ]]; then
    exec_cmd "mv '$CodeDirectory/genesis-contracts/sprintSizes.js' '$CodeDirectory/genesis-contracts/sprintSizes.js.backup'"
  fi

  echo "Generate Bor validator set"
  exec_cmd "cd '$CodeDirectory/genesis-contracts'"
  # Generates the ./code/genesis-contracts/contracts/BorValidatorSet.sol file
  exec_cmd "node generate-borvalidatorset.js --bor-chain-id '$BorChainId' --heimdall-chain-id '$HeimdallChainId'"

  echo "Generate genesis.json"
  # Generates the ./code/genesis-contracts/genesis.json file
  exec_cmd "node generate-genesis.js --bor-chain-id '$BorChainId' --heimdall-chain-id '$HeimdallChainId'"
}

setup_contracts() {
  if [[ ! -d "$CodeDirectory/contracts" ]]; then
    echo "Cloning matic-contracts repository"
    exec_cmd "cd '$CodeDirectory'"
    exec_cmd "git clone 'https://github.com/maticnetwork/contracts.git' --branch '$contractsBranch' contracts"
    exec_cmd "cd '$CodeDirectory/contracts'"
  else
    check_repository "$CodeDirectory/contracts" 'https://github.com/maticnetwork/contracts.git'
    echo "Updating matic-contracts repository"
    exec_cmd "cd '$CodeDirectory/contracts'"
    exec_cmd "git checkout '$contractsBranch'"
  fi
  exec_cmd "npm install --omit=dev"

  echo "Process templates"
  exec_cmd "npm run template:process -- --bor-chain-id '$BorChainId'"

  echo "Compile matic-contracts"
  exec_cmd 'npm run truffle:compile'

  echo "Copy genesis.json file"
  exec_cmd "cp '$BaseDirectory/config/contractAddresses.json' '$CodeDirectory/contracts/contractAddresses.json'"
}

setup_bor() {
  echo "Prepare data directory"
  exec_cmd "cd '$BaseDirectory'"
  exec_cmd "mkdir -p '$BaseDirectory/devnet/bor/keystore'"

  echo "Process template scripts"
  # Generates the ./matic-cli/src/setup/bor/templates/*.njk files

  echo "Prepare keystore and password.txt"
  local primaryAccount="$(jq --raw-output '.[0].address' "$BaseDirectory/devnet/signer-dump.json")"
  local keystoreFilename="UTC--$(date +'%Y-%m-%dT%H:%M:%S%z' | sed -re 's/:/-/gi')--$primaryAccount"
}

build_heimdall
build_bor
#create_heimdall_testnet_files
setup_genesis_contracts
setup_contracts
#setup_bor
