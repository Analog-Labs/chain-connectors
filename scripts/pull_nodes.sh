#!/bin/bash
set -e

docker image pull ethereum/client-go:v1.12.2
docker image pull parity/polkadot:v1.5.0
docker image pull staketechnologies/astar-collator:v5.28.0-rerun
