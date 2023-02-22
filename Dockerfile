FROM rust:1.67-slim-bookworm as base
RUN apt-get update && apt-get install -y libvips42 libvips-dev libc6 openssl libssl-dev libspng-dev libwebp-dev


FROM base as build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY ./.cargo .cargo
COPY ./lib lib
COPY ./src src
RUN cargo install --path .


FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libvips42 libvips-dev libspng-dev libwebp-dev
WORKDIR /usr/app
COPY --from=build /app/target/release .
COPY --from=build /app/lib lib
CMD ["/usr/app/pdf-service"]
