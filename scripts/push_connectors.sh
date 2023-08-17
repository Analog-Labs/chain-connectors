#!/bin/bash
set -e

# First arg is the Docker image tag on all images
tag=$1

docker push analoglabs/connector-bitcoin:latest
docker push analoglabs/connector-ethereum:latest
docker push analoglabs/connector-polkadot:latest
docker push analoglabs/connector-astar:latest

if [[ -n "${tag}" ]]; then
    echo "Tagging all images: ${tag}";
    docker tag analoglabs/connector-bitcoin:latest "analoglabs/connector-bitcoin:${tag}"
    docker push "analoglabs/connector-bitcoin:${tag}"

    docker tag analoglabs/connector-ethereum:latest "analoglabs/connector-ethereum:${tag}"
    docker push "analoglabs/connector-ethereum:${tag}"

    docker tag analoglabs/connector-polkadot:latest "analoglabs/connector-polkadot:${tag}"
    docker push "analoglabs/connector-polkadot:${tag}"

    docker tag analoglabs/connector-astar:latest "analoglabs/connector-astar:${tag}"
    docker push "analoglabs/connector-astar:${tag}"
fi
