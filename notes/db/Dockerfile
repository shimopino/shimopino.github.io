FROM --platform=linux/amd64 mysql:8.0.33-debian

RUN apt-get -y update \
  && apt-get -y install wget
RUN wget https://downloads.mysql.com/docs/sakila-db.tar.gz
RUN tar -xvzf sakila-db.tar.gz
RUN mv sakila-db/sakila-schema.sql docker-entrypoint-initdb.d/1_sakila_schema.sql
RUN mv sakila-db/sakila-data.sql docker-entrypoint-initdb.d/2_sakila_data.sql
RUN rm -rf sakila-db

# CMD [ "/entrypoint.sh" ]