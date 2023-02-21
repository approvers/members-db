FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM scratch

WORKDIR /app
COPY --from=builder /app/target/release/members-db /app/members-db

ENV PORT 8080

ENTRYPOINT [ "/app/target/release/members-db" ]