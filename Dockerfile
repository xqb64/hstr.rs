FROM rust:1.61.0

WORKDIR /app
COPY . .

RUN cargo install --path .

CMD ["hstr-rs"]