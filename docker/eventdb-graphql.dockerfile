FROM node:alpine
LABEL description="Instant high-performance GraphQL API for your PostgreSQL database https://github.com/graphile/postgraphile"

RUN apk add --no-cache git

# Install PostGraphile and plugins
RUN npm install -g postgraphile
RUN npm install -g graphile-contrib/pg-simplify-inflector

EXPOSE 5000
ENTRYPOINT ["postgraphile", "-n", "0.0.0.0"]
