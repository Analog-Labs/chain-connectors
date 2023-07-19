FROM scratch

# metadata
LABEL summary="Analog’s connectors for polkadot network" \
      name="${REGISTRY_PATH}/connector-polkadot" \
      version="${IMAGE_VERSION}" \
      description="Polkadot chain connector" \
      one.analog.image.vendor="Analog.one" \
      one.analog.image.source="https://github.com/Analog-Labs/chain-connectors/blob/${VCS_REF}/\
chains/polkadot/Dockerfile" \
      one.analog.image.revision="${VCS_REF}" \
      one.analog.image.created="${BUILD_DATE}"

COPY bin/rosetta-server-polkadot rosetta-server-polkadot
ENTRYPOINT ["/rosetta-server-polkadot"]
