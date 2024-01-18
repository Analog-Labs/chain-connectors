#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "${SCRIPT_DIR}/../"

RUN_FIX=0
RUN_TESTS=0

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
        *)
        warn "Unknown argument: $1"
        usage
        ;;
    esac
done

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
# LINT_FLAGS='-- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'

if [[ "${RUN_FIX}" == "1" ]]; then
  exec_cmd 'format' 'cargo +nightly fmt --all && dprint fmt'
  # exec_cmd 'clippy fix' 'cargo clippy --fix --allow-dirty --workspace --examples --tests --all-features --exclude playground -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
  # exec_cmd 'clippy --fix' 'cargo clippy --fix --workspace --examples --tests --all-features --allow-dirty -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions -Aclippy::missing_errors_doc'
fi
exec_cmd 'shellcheck' 'shellcheck --enable=all --severity=style ./scripts/*.sh'
exec_cmd 'cargo fmt' 'cargo +nightly fmt --all -- --check'
exec_cmd 'dprint check' 'dprint check'
exec_cmd 'cargo deny' 'cargo deny check'

# exec_cmd 'clippy rosetta-server-astar' 'cargo clippy --locked -p rosetta-server-astar --examples --tests -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
# exec_cmd 'clippy rosetta-server-ethereum' 'cargo clippy --locked -p rosetta-server-ethereum --examples --tests -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
# exec_cmd 'clippy rosetta-server-polkadot' 'cargo clippy --locked -p rosetta-server-polkadot --examples --tests -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
# exec_cmd 'clippy rosetta-client' 'cargo clippy --locked -p rosetta-client --examples --tests -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
exec_cmd 'clippy' 'cargo clippy --locked --workspace --examples --tests --all-features --exclude playground -- -Dwarnings -Dclippy::unwrap_used -Dclippy::expect_used -Dclippy::nursery -Dclippy::pedantic -Aclippy::module_name_repetitions'
# exec_cmd 'build connectors' "${SCRIPT_DIR}/build_connectors.sh"

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
    --exclude rosetta-client
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
