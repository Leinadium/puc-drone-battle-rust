FROM rust:1.51 as build

RUN USER=root cargo new --bin drone-battle
WORKDIR /drone-battle

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/drone-battle*
RUN cargo build --release

FROM scratch

COPY --from=build /drone-battle/target/release/drone-battle .
COPY ./config.json ./config.json

CMD ["./drone-battle config.json"]