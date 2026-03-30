# Multi-stage build for dex scaffold web app.
# Build from the REPO ROOT so templates are accessible:
#   docker build -f webapp/Dockerfile -t dex-scaffold .

# ── Stage 1: Build frontend ─────────────────────────────────────────
FROM node:22-alpine AS frontend-build

WORKDIR /app/webapp
COPY webapp/package.json webapp/package-lock.json* ./
RUN npm ci --ignore-scripts

COPY webapp/ .
RUN npm run build

# ── Stage 2: Production server ───────────────────────────────────────
FROM node:22-alpine AS production

WORKDIR /app

# Copy package files and install production dependencies only
COPY webapp/package.json webapp/package-lock.json* ./
RUN npm ci --omit=dev --ignore-scripts

# Copy built frontend
COPY --from=frontend-build /app/webapp/dist ./dist

# Copy server source
COPY webapp/server/ ./server/

# Copy templates from the repo root (embedded in the container)
COPY templates/ ./templates/

ENV NODE_ENV=production
ENV PORT=8080
ENV TEMPLATES_DIR=/app/templates

EXPOSE 8080

CMD ["npx", "tsx", "server/index.ts"]
