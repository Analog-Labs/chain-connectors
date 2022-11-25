FROM ubuntu:20.04 as builder

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

COPY --from=builder /app/bin/rosetta-server-substrate /app/rosetta-server-substrate

RUN chmod -R 755 /app/*

CMD /app/rosetta-server-substrate
