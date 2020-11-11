# syntax=docker/dockerfile:1.0-experimental
FROM alpine AS builder
RUN apk add --no-cache build-base cargo
WORKDIR /usr/src
RUN USER=root cargo new snekcloud-server
WORKDIR /usr/src/snekcloud-server
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=target \
    cargo build --release
RUN mkdir /tmp/snekcloud
RUN --mount=type=cache,target=target cp target/release/snekcloud-server /tmp/snekcloud/
WORKDIR /tmp/snekcloud
RUN ./snekcloud-server generate-key private_key
RUN timeout 1s ./snekcloud-server || exit 0
RUN cp config/00_default.toml config/10_local.toml
RUN rm private_key

FROM alpine
RUN apk add --no-cache build-base
COPY --from=builder /tmp/snekcloud/snekcloud-server .
COPY --from=builder /tmp/snekcloud/config /
ENTRYPOINT ["/snekcloud-server"]