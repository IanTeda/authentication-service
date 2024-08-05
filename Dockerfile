FROM rust:1 AS chef
# We only pay the installation cost once, it will be cached onwards
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json \
    && apt-get update -y \
    && apt-get install -y --no-install-recommends protobuf-compiler libprotobuf-dev \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY . .
ENV SQLX_OFFLINE=true
# Build our project
RUN cargo build --release --bin authentication_microservice

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# RUN apt-get update -y \
#     && apt-get install -y --no-install-recommends openssl ca-certificates \
#     # Clean up
#     && apt-get autoremove -y \
#     && apt-get clean -y \
#     && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/authentication_microservice authentication
COPY configuration configuration
LABEL org.opencontainers.image.source="https://github.com/ianteda/authentication_microservice"
LABEL org.opencontainers.image.description="A service for handling application authentication and sessions"
LABEL org.opencontainers.image.licenses="GPL-3.0"
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./authentication"]