FROM rust:1.82 as builder
WORKDIR /src

# RUN  --mount=type=cache,target=/var/cache/apk,sharing=locked \
#     apk update \
#     && apk add --no-cache musl-dev openssl-dev libgit2-dev pkgconfig  zlib-dev

COPY . .
RUN cargo install --path .

FROM rust:1.82-slim
LABEL maintainer="jan.moeller0@pm.me"

COPY --from=builder /usr/local/cargo/bin/gitwatch /usr/local/bin/gitwatch

ENTRYPOINT [ "gitwatch" ]
