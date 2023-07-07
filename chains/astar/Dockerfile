FROM analoglabs/builder:latest AS builder
FROM scratch
COPY --from=builder /chain-connectors/bin/rosetta-server-astar rosetta-server-astar
ENTRYPOINT ["/rosetta-server-astar"]
