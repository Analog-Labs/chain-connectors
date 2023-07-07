FROM analoglabs/builder:latest AS builder
FROM scratch
COPY --from=builder /chain-connectors/bin/rosetta-server-bitcoin /rosetta-server-bitcoin
ENTRYPOINT ["/rosetta-server-bitcoin"]
