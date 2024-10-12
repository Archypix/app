# Stage 1 - build
FROM node:22-alpine AS build
WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .
RUN npm run build

# Stage 2 - production
FROM node:22-alpine AS final
WORKDIR /app

COPY --from=build /app/.output ./

ENV HOST=0.0.0.0
ENV PORT=80
EXPOSE 80

CMD ["node", "/app/server/index.mjs"]
