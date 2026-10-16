ARG RUST_VERSION=1.99.0
ARG APP_NAME=rust-backend

FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME

WORKDIR /app

# Install host build dependencies.
RUN apk add --no-cache clang lld musl-dev git

# Build the application.
# Cache mounts keep downloaded crates and compiled dependencies between builds;
# bind mounts avoid copying the sources into the image. The executable is copied
# out of the cache-mounted /app/target before it is unmounted.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
cargo build --locked --release && \
cp ./target/release/$APP_NAME /bin/server



FROM alpine:latest AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=build /bin/server /bin/

EXPOSE 3000

CMD ["/bin/server"]
