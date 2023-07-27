#!/bin/sh

# First arg is the Docker image tag on all images
tag=$1

docker push analoglabs/connector-bitcoin:latest
docker push analoglabs/connector-ethereum:latest
docker push analoglabs/connector-polkadot:latest
docker push analoglabs/connector-astar:latest

if [ ! -z "$tag" ]; then
    echo "Tagging all images: $tag";
    docker tag analoglabs/connector-bitcoin analoglabs/connector-bitcoin:$tag
    docker push analoglabs/connector-bitcoin:$tag

    docker tag analoglabs/connector-ethereum analoglabs/connector-ethereum:$tag
    docker push analoglabs/connector-ethereum:$tag

    docker tag analoglabs/connector-polkadot analoglabs/connector-polkadot:$tag
    docker push analoglabs/connector-polkadot:$tag

    docker tag analoglabs/connector-astar analoglabs/connector-astar:$tag
    docker push analoglabs/connector-astar:$tag
fi

