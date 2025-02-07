FROM node:jod-slim AS frontend-build
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN npm install -g corepack@latest && corepack enable
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/pnpm/store \
     pnpm install --frozen-lockfile
RUN pnpm run build

FROM rust:slim AS backend-build
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
     --mount=type=cache,target=/app/target \
     cargo build --release && mv /app/target/release/celestob .

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=frontend-build /app/www ./www
COPY --from=backend-build /app/celestob .
ENTRYPOINT [ "/app/celestob" ]
