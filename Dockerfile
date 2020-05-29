FROM rust:1.43-slim-stretch AS buildcontainer
WORKDIR /usr/src/asset_parser
COPY . .
RUN cargo build --release



FROM debian:stretch-slim AS appcontainer
WORKDIR /usr/bin/asset_parser
COPY --from=buildcontainer /usr/src/asset_parser/target/release/asset_parser .
CMD ["/usr/bin/asset_parser/asset_parser"]