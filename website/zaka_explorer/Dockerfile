# syntax=docker/dockerfile:1

FROM rust:1-slim-bookworm AS build

RUN rustup target add wasm32-unknown-unknown
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install --locked trunk@0.22.0

WORKDIR /src
COPY . .

# The address the explorer is served from, which its share tags start with:
# pass it with --build-arg.
ARG ZAKA_SITE_ORIGIN

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    --mount=type=cache,target=/root/.cache/trunk \
    --mount=type=secret,id=zaka_pda_font_secret,env=ZAKA_PDA_FONT_SECRET \
    (cd website/zaka_web_ui_common/fonts && trunk build --release) \
    && cd website/zaka_explorer \
    && trunk build --release --locked --cargo-profile wasm-release \
    && cp -r dist /srv/explorer

FROM joseluisq/static-web-server:2 AS run

COPY --from=build /srv/explorer /public
