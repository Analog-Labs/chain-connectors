#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "${SCRIPT_DIR}/../"

# Check for 'uname' and abort if it is not available.
uname -v > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'uname' to identify the platform."; exit 1; }

RUN_FIX=0
RUN_TESTS=0
TEST_ETH_BACKEND=0

# process arguments
while [[ $# -gt 0 ]]
do
    case "$1" in
        --test|--tests)
        RUN_TESTS=1
        shift 1
        ;;
        --fix)
          RUN_FIX=1
          shift 1
        ;;
        --eth-backend)
          TEST_ETH_BACKEND=1
          shift 1
        ;;
        *)
        warn "Unknown argument: $1"
        usage
        ;;
    esac
done

# Check for 'docker' and abort if it is not running.
docker info > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'docker', please start docker and try again."; exit 1; }

# Check for 'cargo' and abort if it is not available.
cargo -V > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'cargo' for compile the binaries"; exit 1; }

# Check for 'solc' and abort if it is not available.
solc --version > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'solc >= 0.8.20' for compile the contracts"; exit 1; }

# Check for 'cargo deny' and abort if it is not available.
cargo deny -V > /dev/null 2>&1 || { echo >&2 "ERROR - 'cargo deny' not found, install using 'cargo install --locked cargo-deny'"; exit 1; }

# Check for 'dprint' and abort if it is not available.
dprint -V > /dev/null 2>&1 || { echo >&2 "ERROR - 'dprint' not found, install using 'cargo install --locked dprint'"; exit 1; }

# Check for 'shellcheck' and abort if it is not available.
shellcheck -V > /dev/null 2>&1 || { echo >&2 "ERROR - 'shellcheck' not found, please visit 'https://github.com/koalaman/shellcheck?tab=readme-ov-file#installing'"; exit 1; }

# Setup console colors
if test -t 1 && command -v tput >/dev/null 2>&1; then
    ncolors=$(tput colors)
    if test -n "${ncolors}" && test "${ncolors}" -ge 8; then
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
  printf >&2 "${bold_color}${warn_color}%s${reset_color}\n${error_color}%s${reset_color}\n" "$1" "$2"
}

# Execute a command and only print it if it fails
exec_cmd() {
  printf '  %s... ' "$1"
  local cmdOutput
  if eval "cmdOutput=\$( { $2 ;} 2>&1 )" > /dev/null; then
    # Success
    echo "${bold_color}${green_color}OK${reset_color}"
  else
    # Failure
    echo "${bold_color}${error_color}FAILED${reset_color}"
    print_failed_command "$2" "${cmdOutput}"
    exit 1
  fi
}

CLIPPY_FLAGS="-Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions"
if [[ "${RUN_FIX}" == "1" ]]; then
  exec_cmd 'format' 'cargo +nightly fmt --all && dprint fmt'
  # exec_cmd 'clippy --fix' "cargo clippy --fix --allow-dirty --workspace --examples --tests --all-features --exclude playground -- ${CLIPPY_FLAGS}"
fi
exec_cmd 'shellcheck' 'shellcheck --enable=all --severity=style ./scripts/*.sh'
exec_cmd 'cargo fmt' 'cargo +nightly fmt --all -- --check'
exec_cmd 'dprint check' 'dprint check'
exec_cmd 'cargo deny' 'cargo deny check'

# Run clippy on all packages with different feature flags
# LINT_FLAGS='-- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'

# exec_cmd 'ethereum build all-features' 'cargo build -p rosetta-config-ethereum --all-features'
# exec_cmd 'ethereum test all-features' 'cargo test -p rosetta-config-ethereum --all-features'
# exec_cmd 'ethereum clippy all-features' "cargo clippy -p rosetta-config-ethereum --all-features ${LINT_FLAGS}"
# ethereumFeatures=('std' 'std,serde' 'std,scale-info' 'std,scale-codec')
# for features in "${ethereumFeatures[@]}";
# do
#   exec_cmd "ethereum build ${features}" "cargo build -p rosetta-config-ethereum --no-default-features --features=${features}"
#   exec_cmd "ethereum test ${features}" "cargo test -p rosetta-config-ethereum --no-default-features --features=${features}"
#   exec_cmd "ethereum clippy ${features}" "cargo clippy -p rosetta-config-ethereum --no-default-features --features=${features} ${LINT_FLAGS}"
# done


# exec_cmd 'clippy rosetta-server-astar' 'cargo clippy --locked -p rosetta-server-astar --examples --tests -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
# exec_cmd 'clippy rosetta-server-ethereum' 'cargo clippy --locked -p rosetta-server-ethereum --examples --tests -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
# exec_cmd 'clippy rosetta-server-polkadot' 'cargo clippy --locked -p rosetta-server-polkadot --examples --tests -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
# exec_cmd 'clippy rosetta-client' 'cargo clippy --locked -p rosetta-client --examples --tests -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
exec_cmd 'clippy' "cargo clippy --locked --workspace --examples --tests --all-features --exclude playground -- ${CLIPPY_FLAGS}"

if [[ "${TEST_ETH_BACKEND}" == "1" ]]; then
  NAME='rosetta-ethereum-backend'
  exec_cmd 'clippy all-features' "cargo clippy -p ${NAME} --tests --all-features -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy no-default-features' "cargo clippy -p ${NAME} --tests --no-default-features -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy std' "cargo clippy -p ${NAME} --tests --no-default-features --features=std -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy serde' "cargo clippy -p ${NAME} --tests --no-default-features --features=serde -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy jsonrpsee' "cargo clippy -p ${NAME} --tests --no-default-features --features=jsonrpsee -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy with-codec' "cargo clippy -p ${NAME} --tests --no-default-features --features=with-codec -- ${CLIPPY_FLAGS}"
  exec_cmd 'build wasm32-unknown-unknown' "cargo build -p ${NAME} --target wasm32-unknown-unknown --no-default-features --features=with-codec,jsonrpsee,serde"
fi

if [[ "${RUN_TESTS}" == "1" ]]; then
  exec_cmd 'cleanup docker' "${SCRIPT_DIR}/reset_docker.sh"
  cargo test --locked -p rosetta-server-ethereum
  cargo test --locked -p rosetta-server-astar
  cargo test --locked -p rosetta-server-polkadot
  cargo test --locked -p rosetta-client
  cargo test --locked --workspace --all-features \
    --exclude rosetta-server-astar \
    --exclude rosetta-server-ethereum \
    --exclude rosetta-server-polkadot \
    --exclude rosetta-client \
    --exclude rosetta-testing-arbitrum
  # cargo test --locked --all-features --workspace
  exec_cmd 'cleanup docker' "${SCRIPT_DIR}/reset_docker.sh"
fi
#exec_cmd 'reset docker' "${SCRIPT_DIR}/reset_docker.sh"
#cargo test --locked --all-features --workspace


#exec_cmd 'cargo test' 'cargo test --locked --all-features --workspace'

#echo "Running ${SCRIPT_DIR}/build_connectors.sh"
#"${SCRIPT_DIR}/build_connectors.sh"
#
#cargo test --locked --all-features --workspace
