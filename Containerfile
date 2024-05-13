FROM node:20 as build

WORKDIR /app

COPY . ./
RUN npm install
RUN npm run build


FROM caddy:2.8-alpine

COPY --from=build /app/build /usr/share/caddy



