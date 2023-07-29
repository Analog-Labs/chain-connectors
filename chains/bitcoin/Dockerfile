ARG REGISTRY_PATH
ARG IMAGE_VERSION
ARG VCS_REF
ARG BUILD_DATE

FROM alpine:latest AS certs
RUN apk --update add ca-certificates

FROM scratch

# metadata
LABEL summary="Analog’s connectors for bitcoin network" \
      name="${REGISTRY_PATH}/connector-bitcoin" \
      version="${IMAGE_VERSION}" \
      description="Bitcoin chain connector" \
      one.analog.image.vendor="Analog One Foundation" \
      one.analog.image.source="https://github.com/Analog-Labs/chain-connectors/blob/${VCS_REF}/\
chains/bitcoin/Dockerfile" \
      one.analog.image.revision="${VCS_REF}" \
      one.analog.image.created="${BUILD_DATE}"

ENV PATH=/bin
COPY bin/rosetta-server-bitcoin /bin/rosetta-server-bitcoin
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# check if executable works in this container
RUN ["/bin/rosetta-server-bitcoin", "--help"]

ENTRYPOINT ["/bin/rosetta-server-bitcoin"]
