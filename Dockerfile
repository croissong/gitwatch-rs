FROM rust:1.92 as builder
WORKDIR /src

RUN apt-get update && apt-get install -y \
    libssl-dev \
    libgit2-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo install --path .



FROM rust:1.92-slim

COPY --from=builder /usr/local/cargo/bin/gitwatch /usr/local/bin/gitwatch

ENTRYPOINT [ "gitwatch" ]
