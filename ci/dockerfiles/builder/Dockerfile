FROM analoglabs/base-ci-linux:latest

WORKDIR /chain-connectors
COPY . /chain-connectors

RUN set -eux && \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) targetArch='x86_64-unknown-linux-musl' ;; \
        armhf) targetArch='armv7-unknown-linux-musleabi' ;; \
        arm64) targetArch='aarch64-unknown-linux-musl' ;; \
        i386) targetArch='i686-unknown-linux-musl' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    cargo build \
      --locked \
      --release \
      --target ${targetArch} \
      -p rosetta-server-bitcoin \
      -p rosetta-server-ethereum \
      -p rosetta-server-polkadot \
      -p rosetta-server-astar && \
    mkdir -p ./bin && \
    mv target/${targetArch}/release/rosetta-server-bitcoin ./bin/rosetta-server-bitcoin && \
    mv target/${targetArch}/release/rosetta-server-ethereum ./bin/rosetta-server-ethereum && \
    mv target/${targetArch}/release/rosetta-server-polkadot ./bin/rosetta-server-polkadot && \
    mv target/${targetArch}/release/rosetta-server-astar ./bin/rosetta-server-astar

ENTRYPOINT ["/chain-connectors/bin/rosetta-server-bitcoin"]
