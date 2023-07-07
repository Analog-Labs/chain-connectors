FROM analoglabs/builder:latest AS builder
FROM scratch
COPY --from=builder /chain-connectors/bin/rosetta-server-polkadot rosetta-server-polkadot
ENTRYPOINT ["/rosetta-server-polkadot"]
