FROM analoglabs/builder:latest AS builder
FROM scratch
COPY --from=builder /chain-connectors/bin/rosetta-server-ethereum rosetta-server-ethereum
ENTRYPOINT ["/rosetta-server-ethereum"]
