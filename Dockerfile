FROM rust:alpine
RUN apk add musl-dev openssl-dev
WORKDIR /site
COPY . .
RUN cargo build --release
RUN mv ./target/release/bobertoyindotcom /usr/local/bin

ENTRYPOINT [ "/usr/local/bin/bobertoyindotcom" ]
