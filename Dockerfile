FROM rust:latest as build

# create a new empty shell project
RUN USER=root cargo new --bin bobertoyindotcom
WORKDIR /bobertoyindotcom

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/bobertoyin*
RUN cargo build --release

# our final base
FROM debian:stable-slim
WORKDIR /site

# copy the build artifact from the build stage
COPY --from=build /bobertoyindotcom/target/release/bobertoyindotcom .
COPY content ./content
COPY static ./static
COPY templates ./templates

# set the startup command to run your binary
CMD ["./bobertoyindotcom"]
