# 멀티스테이지 빌드로 최적화
FROM rust:1.75 as builder

WORKDIR /app

# 의존성 캐싱을 위해 먼저 복사
COPY Cargo.toml Cargo.lock ./

# 빈 src 생성하여 의존성만 빌드
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 실제 소스 복사 및 빌드
COPY src ./src
RUN cargo build --release

# 런타임 이미지 (경량화)
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/p2p-game-server /usr/local/bin/

EXPOSE 4001 8080

CMD ["p2p-game-server"]