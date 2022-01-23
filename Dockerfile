FROM rust:1.57 as builder

RUN cargo new --bin rusty-slackbot
WORKDIR ./rusty-slackbot
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/rusty*
RUN cargo build --release


FROM debian:buster-slim

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 443
RUN mkdir -p /home/rusty-slackbot
COPY --from=builder /rusty-slackbot/target/release/rusty /home/rusty-slackbot
RUN chmod +x /home/rusty-slackbot

WORKDIR /home/rusty-slackbot

CMD ["./rusty"]
