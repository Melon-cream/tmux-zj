FROM rust:1-slim-bookworm

RUN apt-get update -qq && apt-get install -y -qq --no-install-recommends \
        tmux python3 python3-pyte ca-certificates musl-tools gcc \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl && rustup component add rustfmt clippy

WORKDIR /w
