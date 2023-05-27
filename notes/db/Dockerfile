FROM --platform=linux/amd64 mysql:8.0.33-debian

RUN apt-get -y update && \
    apt-get -y install wget && \
    wget https://downloads.mysql.com/docs/sakila-db.tar.gz && \
    tar -xvzf sakila-db.tar.gz && \
    rm sakila-db.tar.gz && \
    mv sakila-db/sakila-schema.sql docker-entrypoint-initdb.d/1_sakila_schema.sql && \
    mv sakila-db/sakila-data.sql docker-entrypoint-initdb.d/2_sakila_data.sql && \
    rm -rf sakila-db && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# CMD [ "/entrypoint.sh" ]