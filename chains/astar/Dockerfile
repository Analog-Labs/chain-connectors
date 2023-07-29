ARG REGISTRY_PATH
ARG IMAGE_VERSION
ARG VCS_REF
ARG BUILD_DATE

#FROM lohann/builder:1.0.0 as builder
#
#WORKDIR /builds
#
#COPY . .
#
#RUN set -eux; \
#    apt-get -y update; \
#	dpkgArch="$(dpkg --print-architecture)"; \
#    apt-get install -y --no-install-recommends musl-dev musl-tools && \
#    case "${dpkgArch##*-}" in \
#        amd64) rustArch='x86_64-unknown-linux-gnu'; rustTargetArch='x86_64-unknown-linux-musl' ;; \
#        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustTargetArch='armv7-unknown-linux-musleabi' ;; \
#        arm64) rustArch='aarch64-unknown-linux-gnu'; rustTargetArch='aarch64-unknown-linux-musl' ;; \
#        i386) rustArch='i686-unknown-linux-gnu'; rustTargetArch='i686-unknown-linux-musl' ;; \
#        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
#    esac; \
#    rustup target add "$rustTargetArch" && \
#    cargo build --release -p rosetta-server-astar --target "$rustTargetArch" && \
#    cp target/$rustTargetArch/release/rosetta-server-astar /builds/target/release/rosetta-server-astar

FROM alpine:latest as certs
RUN apk --update add ca-certificates

FROM scratch

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
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
#COPY --from=builder /builds/target/release/rosetta-server-astar /bin/rosetta-server-astar
COPY bin/rosetta-server-astar /bin/rosetta-server-astar

ENV RUST_LOG="debug,subxt=debug,jsonrpsee-client-transport=debug"

# check if executable works in this container
RUN ["/bin/rosetta-server-astar", "--help"]

ENTRYPOINT ["/bin/rosetta-server-astar"]
