FROM alpine:latest AS certs

ARG BUILD_ID="astar-certs"
LABEL stage="certs" \
      build="$BUILD_ID"

RUN apk --update add ca-certificates

FROM scratch

ARG REGISTRY_PATH
ARG IMAGE_VERSION
ARG VCS_REF
ARG BUILD_DATE

# metadata
LABEL summary="Analog’s connectors for astar parachain" \
      name="${REGISTRY_PATH}/connector-astar" \
      version="${IMAGE_VERSION}" \
      description="Astar chain connector" \
      one.analog.image.vendor="Analog One Foundation" \
      one.analog.image.source="https://github.com/Analog-Labs/chain-connectors/blob/${VCS_REF}/\
chains/astar/Dockerfile" \
      one.analog.image.revision="${VCS_REF}" \
      one.analog.image.created="${BUILD_DATE}"

ENV PATH=/bin
COPY bin/rosetta-server-astar /bin/rosetta-server-astar
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# check if executable works in this container
RUN ["/bin/rosetta-server-astar", "--help"]

ENTRYPOINT ["/bin/rosetta-server-astar"]
