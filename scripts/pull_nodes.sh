#!/bin/bash
set -e

docker image pull ruimarinho/bitcoin-core:23
docker image pull ethereum/client-go:v1.12.2
docker image pull parity/polkadot:v1.0.0
docker image pull staketechnologies/astar-collator:v5.15.0
