# Stage 1 — build environment (with /bin/sh)
FROM alpine:3.20
RUN apk add --no-cache cloudflared

# Stage 2 — lightweight Cloudflare image
FROM cloudflare/cloudflared:latest
WORKDIR /etc/cloudflared
ENTRYPOINT ["cloudflared", "tunnel", "run", "hermesai"]
