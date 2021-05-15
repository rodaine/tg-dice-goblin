FROM rust:latest as build

COPY ./ ./
RUN cargo build --release
RUN mkdir -p /build-out
RUN cp target/release/tg-dice-goblin /build-out/

FROM debian:buster-slim

RUN DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get -y install ca-certificates libssl-dev dumb-init \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /build-out/tg-dice-goblin /
ENTRYPOINT [ "/usr/bin/dumb-init", "--" ]
CMD [ "/tg-dice-goblin" ]
