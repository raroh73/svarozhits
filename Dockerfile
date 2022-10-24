FROM --platform=$BUILDPLATFORM rust:1.64.0 AS builder
RUN apt-get update \
    && apt-get install -y gcc-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*
ARG TARGETPLATFORM
RUN echo "Setting cargo target for $TARGETPLATFORM" && \
    case "$TARGETPLATFORM" in \
      linux/amd64) \
        TARGET="x86_64-unknown-linux-gnu"; \
        break;; \
      linux/arm64) \
        TARGET="aarch64-unknown-linux-gnu"; \
        break;; \
      *) echo "Unsupported platform: ${TARGETPLATFORM}!"; exit 1;; \
    esac && \
    echo "$TARGET" > /tmp/target
RUN rustup target add "$(cat /tmp/target)"
WORKDIR /svarozhits
COPY .cargo/ .cargo/
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build --target "$(cat /tmp/target)" --release
COPY . .
RUN touch src/main.rs && \
    cargo build --target "$(cat /tmp/target)" --release && \
    mv target/"$(cat /tmp/target)"/release/svarozhits /tmp/svarozhits


FROM --platform=$TARGETPLATFORM debian:11.5-slim AS runtime
COPY --from=builder /tmp/svarozhits /usr/local/bin/
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/svarozhits"]
