# The image a game is played in: its own virtual display and the minigame binary, pinned by
# Cargo.lock.

FROM docker.io/library/rust:1.99-slim-trixie AS game-build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN cargo build --release --locked -p minigame-cli

FROM docker.io/library/debian:trixie-slim AS game
RUN apt-get update \
 && apt-get install -y --no-install-recommends xvfb xkb-data \
 && rm -rf /var/lib/apt/lists/*
COPY --from=game-build /src/target/release/minigame /usr/local/bin/minigame
ENTRYPOINT ["minigame"]
