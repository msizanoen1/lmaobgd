FROM rust:1.44 AS build

RUN apt-get update && \
    apt-get install -y libpq-dev

RUN cargo install \
        diesel_cli \
        --no-default-features \
        --features postgres \
        --version 1.4.0

WORKDIR /usr/src/lmaobgd
COPY . .

RUN cargo install --path . && cargo clean

FROM debian:buster

RUN apt-get update && \
    apt-get install -y libpq5 postgresql-client

COPY --from=build \
     /usr/local/cargo/bin/diesel \
     /usr/local/cargo/bin/lmaobgd \
     /usr/local/cargo/bin/lmaoctl \
     /usr/local/bin/

WORKDIR /usr/src/lmaobgd

COPY migrations migrations

CMD lmaobgd
