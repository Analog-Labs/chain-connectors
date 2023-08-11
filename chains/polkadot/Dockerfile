FROM alpine:latest AS certs

ARG BUILD_ID="polkadot-certs"
LABEL stage="certs" \
      build="$BUILD_ID"

RUN apk --update add ca-certificates

FROM scratch

ARG REGISTRY_PATH
ARG IMAGE_VERSION
ARG VCS_REF
ARG BUILD_DATE

# metadata
LABEL summary="Analog’s connectors for polkadot network" \
      name="${REGISTRY_PATH}/connector-polkadot" \
      version="${IMAGE_VERSION}" \
      description="Polkadot chain connector" \
      one.analog.image.vendor="Analog One Foundation" \
      one.analog.image.source="https://github.com/Analog-Labs/chain-connectors/blob/${VCS_REF}/\
chains/polkadot/Dockerfile" \
      one.analog.image.revision="${VCS_REF}" \
      one.analog.image.created="${BUILD_DATE}"

ENV PATH=/bin
COPY bin/rosetta-server-polkadot /bin/rosetta-server-polkadot
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# check if executable works in this container
RUN ["/bin/rosetta-server-polkadot", "--help"]

ENTRYPOINT ["/bin/rosetta-server-polkadot"]
