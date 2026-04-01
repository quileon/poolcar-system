FROM docker.io/oven/bun:1-alpine AS runtime
ARG PUBLIC_API_BASE_URL=http://localhost:8080
ARG PUBLIC_WS_BASE_URL=ws://localhost:3000
ENV PUBLIC_API_BASE_URL=${PUBLIC_API_BASE_URL}
ENV PUBLIC_WS_BASE_URL=${PUBLIC_WS_BASE_URL}
WORKDIR /app
COPY package.json bun.lock /app/
RUN bun install --frozen-lockfile
COPY . /app/
RUN bun run build
EXPOSE 3000
CMD ["bun", "./build"]
