FROM postgres:15-alpine

ENV POSTGRES_USER=postgres \
    POSTGRES_PASSWORD=postgres \
    POSTGRES_DB=dropcode

EXPOSE 5432
VOLUME ["/var/lib/postgresql/data"]

COPY init.sql /docker-entrypoint-initdb.d/