FROM rust:1.66 as builder

# build rosetta
COPY . src
RUN cd src && cargo install --path bitcoin/server --root /app

# final image
FROM ubuntu:20.04

RUN apt-get update && apt-get install

RUN mkdir -p /app \
    && chown -R nobody:nogroup /app \
    && mkdir -p /data \
    && chown -R nobody:nogroup /data

WORKDIR /app

COPY --from=builder /app/bin/rosetta-server-bitcoin /app/rosetta-server-bitcoin

RUN chmod -R 755 /app/*

ENTRYPOINT ["/app/rosetta-server-bitcoin"]
