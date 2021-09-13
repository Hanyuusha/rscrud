FROM rust:1.54

ENV CARGO_TERM_COLOR always
RUN apt-get update && apt-get install -y libpq-dev
RUN rustup component add rustfmt

WORKDIR /app/
COPY ./ ./
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libpq-dev
COPY --from=0 /app/target/release/rust_t /usr/local/bin/server
CMD ["server"]