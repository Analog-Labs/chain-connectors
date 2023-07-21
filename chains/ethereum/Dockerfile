ARG REGISTRY_PATH
ARG IMAGE_VERSION
ARG VCS_REF
ARG BUILD_DATE

FROM scratch

# metadata
LABEL summary="Analog’s connectors for ethereum network" \
      name="${REGISTRY_PATH}/connector-ethereum" \
      version="${IMAGE_VERSION}" \
      description="Ethereum chain connector" \
      one.analog.image.vendor="Analog One Foundation" \
      one.analog.image.source="https://github.com/Analog-Labs/chain-connectors/blob/${VCS_REF}/\
chains/ethereum/Dockerfile" \
      one.analog.image.revision="${VCS_REF}" \
      one.analog.image.created="${BUILD_DATE}"

COPY bin/rosetta-server-ethereum rosetta-server-ethereum
ENTRYPOINT ["/rosetta-server-ethereum"]
