#!/bin/bash
set -e

# Docker built-in networks
dockerNetworksWhitelist=(
  "$(docker network inspect bridge --format='{{ .Id }}')"
  "$(docker network inspect host --format='{{ .Id }}')"
  "$(docker network inspect none --format='{{ .Id }}')"
)

# Stop all containers
dockerContainers=()
while IFS='' read -r line; do dockerContainers+=("$line"); done < <(docker ps -a -q)
if [[ "${#dockerContainers[@]}" -gt 0 ]]; then
  docker stop "${dockerContainers[@]}"
fi

# Remove all containers
dockerContainers=()
while IFS='' read -r line; do dockerContainers+=("$line"); done < <(docker ps -a -q)
if [[ "${#dockerContainers[@]}" -gt 0 ]]; then
  docker rm "${dockerContainers[@]}"
fi

# Remove all volumes
dockerVolumes=()
while IFS='' read -r line; do dockerVolumes+=("$line"); done < <(docker volume ls -q)
if [[ "${#dockerVolumes[@]}" -gt 0 ]]; then
  docker volume rm "${dockerVolumes[@]}"
fi

# List all docker networks
dockerNetworks=()
while IFS='' read -r line; do dockerNetworks+=("$line"); done < <(docker network ls -q --no-trunc)

# Skip whitelisted networks
for index in "${!dockerNetworks[@]}";
do
  if echo "${dockerNetworksWhitelist[@]}" | grep -w -q "${dockerNetworks[$index]}"; then
    unset -v 'dockerNetworks[$index]'
  fi
done

# Remove all networks, except the built-in ones
if [[ "${#dockerNetworks[@]}" -gt 0 ]]; then
  docker network rm "${dockerNetworks[@]}"
fi
