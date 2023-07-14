#!/usr/bin/env sh
set -x

# private key to deploy contracts
export PRIVATE_KEY=0xd1cce67d8b7a6a6e4a6e9663810c0e3d6f2b129e9f9c1c7a3a0329a0c866bd18
export MNEMONIC=0xd1cce67d8b7a6a6e4a6e9663810c0e3d6f2b129e9f9c1c7a3a0329a0c866bd18

# export heimdall id
export HEIMDALL_ID=heimdall-15001

# Start all nodes
docker compose up -d

# Wait for bor to start
while true
do
    peers=$(docker exec bor0 bash -c "bor attach /root/bor.ipc -exec 'admin.peers'")
    block=$(docker exec bor0 bash -c "bor attach /root/bor.ipc -exec 'eth.blockNumber'")

    if [[ -n "$peers" ]] && [[ -n "$block" ]]; then
        break
    fi
done

echo "$peers"
echo "$block"

# cd matic contracts repo
cd ./code/contracts || exit 1

# bor contracts are deployed on child chain
npm run truffle:migrate:dev:bor -- --reset -f 5 --to 5 || exit 1

# root contracts are deployed on base chain
npm run truffle:migrate:dev -- -f 6 --to 6 || exit 1
