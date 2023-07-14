#!/usr/bin/env sh

docker compose -f ./docker-compose.yml down -v
rm -rf config data devnet logs

