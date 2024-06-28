FROM node:20 AS build

WORKDIR /app


# copy project file
COPY package.json package-lock.json ./
COPY ./scripts ./scripts
COPY ./public ./public

# install all deps (vite is needed when building, which is a dev dependency)
RUN npm install

COPY . ./
RUN npm run build


FROM caddy:2.8-alpine

COPY --from=build /app/build /usr/share/caddy



