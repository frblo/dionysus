# == Builder Stage ==
FROM node:25.6.0-alpine AS builder
WORKDIR /usr/src/dionysus/frontend

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

# == Production Stage ==
FROM docker.io/library/nginx:alpine
WORKDIR /usr/share/nginx/html

RUN rm -rf ./*

COPY --from=builder /usr/src/dionysus/frontend/build .

COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 3000

CMD ["nginx", "-g", "daemon off;"]
