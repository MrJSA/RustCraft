FROM node:22-alpine AS builder

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .
RUN npm run build
RUN npm prune --production

FROM node:22-alpine

WORKDIR /app

COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/server.mjs ./server.mjs
COPY --from=builder /app/node_modules ./node_modules

# Run without root rights (the image ships the unprivileged "node" user)
USER node

ENV PORT=4000
ENV NODE_ENV=production

EXPOSE 4000

CMD ["node", "server.mjs"]
