# ==========================
# Stage 1 — Build the assets
# ==========================
FROM node:20-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build


# ==========================================
# Stage 2 — Serve static files with Nginx
# ==========================================
FROM nginx:alpine

# Optional: overwrite default nginx.conf if you have one
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy built assets from the previous stage
COPY --from=builder /app/dist/dropcode-client/browser /usr/share/nginx/html

EXPOSE 80
