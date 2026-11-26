# Two images of zakac, each of which runs zakac itself:
#
#   docker build -t zaka .                   builds zakac from this source tree
#   docker build --target release …          copies in the zakac of a release
#
# The release workflow builds the second for each Linux CPU and pushes it to
# ghcr.io; it unpacks the executable of each CPU to dist/docker/<arch>/zakac
# first, named as Docker names the CPU (amd64, arm64).

FROM rust:1-slim-bookworm AS build

WORKDIR /src
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked --bin zakac \
    && cp target/release/zakac /usr/local/bin/zakac

FROM debian:bookworm-slim AS release

ARG TARGETARCH
COPY dist/docker/${TARGETARCH}/zakac /usr/local/bin/zakac

CMD ["--help"]
ENTRYPOINT ["zakac"]

FROM debian:bookworm-slim AS run

COPY --from=build /usr/local/bin/zakac /usr/local/bin/zakac

CMD ["--help"]
ENTRYPOINT ["zakac"]
