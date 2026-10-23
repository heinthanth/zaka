FROM rust:1-slim-bookworm AS build

WORKDIR /src
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked --bin zakac \
    && cp target/release/zakac /usr/local/bin/zakac

FROM debian:bookworm-slim AS run

COPY --from=build /usr/local/bin/zakac /usr/local/bin/zakac
RUN useradd --uid 10001 --no-create-home zaka
USER zaka

CMD ["--help"]
ENTRYPOINT ["zakac"]
