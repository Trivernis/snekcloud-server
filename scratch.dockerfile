FROM alpine AS builder
RUN apk add --no-cache build-base cargo
WORKDIR /usr/src
RUN USER=root cargo new snekcloud-server
WORKDIR /usr/src/snekcloud-server
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release
WORKDIR target/release/
RUN ./snekcloud-server generate-key private_key
RUN timeout 1s ./snekcloud-server || exit 0
RUN cp config/00_default.toml config/10_local.toml

FROM scratch
COPY --from=builder /usr/src/snekcloud-server/target/release/snekcloud-server .
COPY --from=builder /usr/src/snekcloud-server/target/release/config /
COPY --from=builder /usr/src/snekcloud-server/target/release/private_key /
ENTRYPOINT ["/snekcloud-server"]