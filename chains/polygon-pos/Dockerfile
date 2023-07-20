FROM scratch

# metadata
LABEL summary="Analog’s connectors for polygon network" \
      name="${REGISTRY_PATH}/connector-polygon" \
      version="${IMAGE_VERSION}" \
      description="Polygon PoS chain connector" \
      one.analog.image.vendor="Analog One Foundation" \
      one.analog.image.source="https://github.com/Analog-Labs/chain-connectors/blob/${VCS_REF}/\
chains/polygon-pos/Dockerfile" \
      one.analog.image.revision="${VCS_REF}" \
      one.analog.image.created="${BUILD_DATE}"

COPY bin/rosetta-server-polygon-pos rosetta-server-polygon-pos
ENTRYPOINT ["/rosetta-server-polygon-pos"]
