FROM analoglabs/builder:latest AS builder
FROM scratch
COPY --from=builder /chain-connectors/bin/rosetta-server-polygon-pos rosetta-server-polygon-pos
ENTRYPOINT ["/rosetta-server-polygon-pos"]
