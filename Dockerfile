FROM rust:slim-buster as builder
RUN apt-get -y update; \
    apt-get install -y --no-install-recommends \
    curl libssl-dev make clang-11 g++ llvm \
    pkg-config libz-dev zstd git; \
    apt-get autoremove -y; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

WORKDIR /slot
COPY . .
RUN cargo build --release --config net.git-fetch-with-cli=true

FROM debian:buster-slim
LABEL description="Slot is a toolchain for rapidly spinning up managed Katana and Torii instances. Play test your game in seconds." \
    authors="tarrence <tarrence@cartridge.gg>" \
    source="https://github.com/cartridge-gg/slot" \
    documentation="https://github.com/cartridge-gg/slot"

RUN apt-get -y update; \
    apt-get install -y --no-install-recommends \
        curl; \
    apt-get autoremove -y; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /slot/target/release/slot /usr/local/bin/slot

