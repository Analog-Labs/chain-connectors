FROM ubuntu:20.04 as builder

# prepare dir
RUN mkdir -p /app \
    && chown -R nobody:nogroup /app
WORKDIR /app

# install deps
RUN apt-get update && apt-get install -y clang curl make protobuf-compiler

# install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- \
    -y --default-toolchain nightly --profile minimal --target wasm32-unknown-unknown
ENV PATH="${PATH}:/root/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin"

# build substrate
RUN cargo install polkadot --git https://github.com/paritytech/polkadot --root /app --tag v0.9.32

# build rosetta
COPY . src
RUN cd src && cargo install --path rosetta-server-substrate --root /app

# final image
FROM ubuntu:20.04

RUN apt-get update && apt-get install

RUN mkdir -p /app \
    && chown -R nobody:nogroup /app \
    && mkdir -p /data \
    && chown -R nobody:nogroup /data

WORKDIR /app

COPY --from=builder /app/bin/polkadot /app/polkadot
COPY --from=builder /app/bin/rosetta-server-substrate /app/rosetta-server-substrate

RUN chmod -R 755 /app/*

CMD /app/rosetta-server-substrate
