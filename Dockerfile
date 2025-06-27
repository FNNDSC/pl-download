# This Dockerfile is only used for *manual* testing.
# It is *not* used for building.

FROM rust:1.88.0-slim-bookworm AS builder
ARG CARGO_TERM_COLOR=always
WORKDIR /usr/local/src/pl-download
COPY Cargo.toml Cargo.lock ./
COPY src ./src/
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /usr/local/src/pl-download/target/release/download /bin/download
CMD ["/bin/download"]
