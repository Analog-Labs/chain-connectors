#!/bin/bash
set -e
shopt -s inherit_errexit

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "${SCRIPT_DIR}/../"

CHECK_FORMAT=0
RUN_FIX=0
RUN_TESTS=0
TEST_ETH_BACKEND=0
TEST_ETH_TYPES=0

# process arguments
while [[ $# -gt 0 ]]
do
    case "$1" in
        --all)
        CHECK_FORMAT=1
        RUN_FIX=1
        RUN_TESTS=1
        TEST_ETH_BACKEND=1
        TEST_ETH_TYPES=1
        shift 1
        ;;
        --fmt|--format)
        CHECK_FORMAT=1
        shift 1
        ;;
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
        --eth-types)
        TEST_ETH_TYPES=1
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

# Check for 'cargo deny' and abort if it is not available.
cargo deny -V > /dev/null 2>&1 || { echo >&2 "ERROR - 'cargo deny' not found, install using 'cargo install --locked cargo-deny'"; exit 1; }

# Check for 'awk' command and abort if it is not available.
command -v awk > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'awk' for verify semantic versioning"; exit 1; }

# Check for 'grep' command and abort if it is not available.
command -v grep > /dev/null 2>&1 || { echo >&2 "ERROR - this script requires 'grep' internally"; exit 1; }

# Check for 'head' command and abort if it is not available.
command -v head > /dev/null 2>&1 || { echo >&2 "ERROR - this script requires 'head' internally"; exit 1; }

# Check for 'solc' and abort if it is not available.
solc --version > /dev/null 2>&1 || { echo >&2 "ERROR - requires 'solc >= 0.8.25' for compile the contracts"; exit 1; }

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

# shellcheck disable=SC2001
function semver2int() {
  local RE='[^0-9]*\([0-9]*\)[.]\([0-9]*\)[.]\([0-9]*\)\([0-9A-Za-z-]*\)'
  local major;
  local minor;
  local patch;
  major="$(echo "$1" | sed -e "s#${RE}#\1#")"
  minor="$(echo "$1" | sed -e "s#${RE}#\2#")"
  patch="$(echo "$1" | sed -e "s#${RE}#\3#")"
  # Convert semver to decimal
  major=$((major * 1000000000))
  minor=$((minor * 1000000))
  patch=$((patch * 1))
  echo $((major + minor + patch))
}

# shellcheck disable=SC2001
function parse_semver() {
  local version="$1"
  if [[ "${version}" =~ ^[0-9]+\.[0-9]+$ ]]; then
    version="${version}.0"
  fi
  local RE='[^0-9]*\([0-9]*\)[.]\([0-9]*\)[.]\([0-9]*\)\([0-9A-Za-z-]*\)'
  local major;
  local minor;
  local patch;
  major="$(echo "${version}" | sed -e "s#${RE}#\1#")"
  minor="$(echo "${version}" | sed -e "s#${RE}#\2#")"
  patch="$(echo "${version}" | sed -e "s#${RE}#\3#")"
  echo "${major}.${minor}.${patch}"
}

# Check sematic version
function checkSemver() {
    local name=''
    local actual
    local expect
    local lhs
    local rhs
    if [[ "$#" -gt 2 ]]; then
        name="${bold_color}${1}${reset_color}${bold_color}:${reset_color} "
        shift 1
    fi
    actual="$(parse_semver "${1}")"
    expect="$(parse_semver "${2}")"
    lhs="$(semver2int "${actual}")"
    rhs="$(semver2int "${expect}")"
    if [[ "${lhs}" -lt "${rhs}" ]]; then
      printf "%s${error_color}%s < %s ${reset_color}\n" "${name}" "${actual}" "${expect}"
      # print suggestion if provided
      if [[ -n "${3}"  ]]; then
        printf "%s\n" "${3}"
      fi
      exit 1
    fi
}

# Check `solc` version
solc_version="$(solc --version)"
solc_version="$(tr '\n' ' ' <<< "${solc_version}")"
solc_version="$(awk -FVersion: '{ print $NF }' <<< "${solc_version}")"
solc_version="$(awk -F. '{print $1 "." $2 "." $3 }' <<< "${solc_version}")"
# shellcheck disable=SC2001
solc_version="$(sed 's/[^0-9\.]*//g' <<< "${solc_version}")"
checkSemver 'solc' \
  "${solc_version}" \
  '0.8.25' \
  "install svm following this instructions: ${bold_color}https://github.com/alloy-rs/svm-rs${reset_color}"

# Check `rustc` version
rustc_version="$(rustc --version)"
rustc_version="$(awk '{print $2}' <<< "${rustc_version}")"
checkSemver 'rustc' \
  "${rustc_version}" \
  '1.79.0' \
  "upgrade rustc with command: ${warn_color}rustup update stable${reset_color}"

# Check `cargo-deny` version
cargo_deny_version="$(cargo deny --version)"
cargo_deny_version="$(awk '{print $2}' <<< "${cargo_deny_version}")"
checkSemver 'cargo deny' \
  "${cargo_deny_version}" \
  '0.14.24' \
  "upgrade ${bold_color}cargo-deny${reset_color} with command: ${warn_color}cargo install cargo-deny${reset_color}"

# Check `dprint` version
dprint_version="$(dprint --version)"
dprint_version="$(awk '{print $2}' <<< "${dprint_version}")"
checkSemver 'dprint' \
  "${dprint_version}" \
  '0.47.1' \
  "upgrade ${bold_color}dprint${reset_color} with command: ${warn_color}cargo install dprint${reset_color}"

# Check `shellcheck` version
shellcheck_version="$(shellcheck --version)"
shellcheck_version="$(grep 'version:' <<< "${shellcheck_version}")"
shellcheck_version="$(awk '{print $2}' <<< "${shellcheck_version}")"
# "$(shellcheck --version | grep 'version:' | awk '{print $2}')"
checkSemver 'shellcheck' \
  "${shellcheck_version}" \
  '0.9.0' \
  "upgrade ${bold_color}shellcheck${reset_color} following the instructions at: ${warn_color}https://github.com/koalaman/shellcheck${reset_color}"

# Check `docker api` version
docker_version="$(docker version)"
docker_version="$(grep 'API version:' <<< "${docker_version}")"
docker_version="$(head -1 <<< "${docker_version}")"
docker_version="$(awk '{ print $3 }' <<< "${docker_version}")"
checkSemver 'docker api' \
  "${docker_version}" \
  '1.42.0' \
  "the minimal docker api version supported by ${bold_color}rosetta-docker${reset_color} is ${warn_color}1.42${reset_color}"

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
  # exec_cmd 'clippy --fix' "cargo clippy --fix --allow-dirty --workspace --examples --tests --all-features -- ${CLIPPY_FLAGS}"
fi

if [[ "${CHECK_FORMAT}" == "1" ]]; then
  exec_cmd 'shellcheck' 'shellcheck --enable=all --severity=style ./scripts/*.sh'
  exec_cmd 'cargo fmt' 'cargo +nightly fmt --all -- --check'
  exec_cmd 'dprint check' 'dprint check'
  exec_cmd 'cargo deny' 'cargo deny check'
  exec_cmd 'clippy' "cargo clippy --locked --workspace --examples --tests --all-features -- ${CLIPPY_FLAGS}"
fi

if [[ "${TEST_ETH_BACKEND}" == "1" ]]; then
  NAME='rosetta-ethereum-backend'
  exec_cmd 'clippy all-features' "cargo --locked clippy -p ${NAME} --tests --all-features -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy no-default-features' "cargo --locked clippy -p ${NAME} --tests --no-default-features -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy std' "cargo clippy --locked -p ${NAME} --tests --no-default-features --features=std -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy serde' "cargo clippy --locked -p ${NAME} --tests --no-default-features --features=serde -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy jsonrpsee' "cargo clippy --locked -p ${NAME} --tests --no-default-features --features=jsonrpsee -- ${CLIPPY_FLAGS}"
  exec_cmd 'clippy with-codec' "cargo clippy --locked -p ${NAME} --tests --no-default-features --features=with-codec -- ${CLIPPY_FLAGS}"
  exec_cmd 'build wasm32-unknown-unknown' "cargo build --locked -p ${NAME} --target wasm32-unknown-unknown --no-default-features --features=with-codec,jsonrpsee,serde"
fi

if [[ "${TEST_ETH_TYPES}" == "1" ]]; then
  NAME='rosetta-ethereum-types'
  # Combine all features, to make sure any combination of features works.
  # The following features must work on wasm32-unknown-unknown targets, once they must be used in substrate runtime.
  FEATURES=(
    '--features=serde'
    '--features=with-rlp'
    '--features=with-codec'
    '--features=with-crypto'
    '--features=serde,with-rlp'
    '--features=serde,with-codec'
    '--features=serde,with-crypto'
    '--features=with-rlp,with-codec'
    '--features=with-rlp,with-crypto'
    '--features=with-rlp,with-codec,with-crypto,serde'
  )

  # all features
  extraflags='--all-features'
  exec_cmd "clippy ${extraflags}" "cargo --locked clippy -p ${NAME} ${extraflags} --tests -- ${CLIPPY_FLAGS}"
  exec_cmd "build ${extraflags}" "cargo --locked build -p ${NAME} ${extraflags}"

  # no features
  extraflags='--no-default-features'
  exec_cmd "clippy ${extraflags}" "cargo --locked clippy -p ${NAME} ${extraflags} --tests -- ${CLIPPY_FLAGS}"
  exec_cmd "build ${extraflags}" "cargo --locked build -p ${NAME} ${extraflags}"
  exec_cmd "build --target wasm32-unknown-unknown ${extraflags}" "cargo build --locked -p ${NAME} --target wasm32-unknown-unknown ${extraflags}"

  # only std feature
  extraflags='--no-default-features --features=std'
  exec_cmd "clippy ${extraflags}" "cargo clippy --locked -p ${NAME} ${extraflags} --tests -- ${CLIPPY_FLAGS}"
  exec_cmd "build ${extraflags}" "cargo build --locked -p ${NAME} ${extraflags}"

  # iterate over all features
  for index in "${!FEATURES[@]}";
  do
    extraflags="${FEATURES[${index}]}"
    exec_cmd "clippy ${extraflags}" "cargo clippy --locked -p ${NAME} --no-default-features ${extraflags} --tests -- ${CLIPPY_FLAGS}"
    exec_cmd "build --target wasm32-unknown-unknown ${extraflags}" "cargo build --locked -p ${NAME} --target wasm32-unknown-unknown --no-default-features ${extraflags}"
  done
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
    --exclude rosetta-testing-arbitrum \
    --exclude rosetta-testing-binance
  #exec_cmd 'cargo test' 'cargo test --locked --all-features --workspace'
  exec_cmd 'cleanup docker' "${SCRIPT_DIR}/reset_docker.sh"
fi
