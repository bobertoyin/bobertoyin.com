FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /site

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /site/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin bobertoyindotcom

FROM gcr.io/distroless/cc-debian12:latest AS runtime
WORKDIR /site
COPY --from=builder /site/target/release/bobertoyindotcom /usr/local/bin
ENTRYPOINT ["/usr/local/bin/bobertoyindotcom"]
