FROM --platform=linux/amd64 mysql:8.0.33-debian

RUN apt-get -y update && \
    apt-get -y install wget git && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# sakilaデータセットを構築する
RUN wget https://downloads.mysql.com/docs/sakila-db.tar.gz && \
    tar -xvzf sakila-db.tar.gz && \
    rm sakila-db.tar.gz && \
    mv sakila-db/sakila-schema.sql docker-entrypoint-initdb.d/1_sakila_schema.sql && \
    mv sakila-db/sakila-data.sql docker-entrypoint-initdb.d/2_sakila_data.sql && \
    rm -rf sakila-db

RUN git clone https://github.com/datacharmer/test_db.git

RUN echo "change source path" && \
    mv test_db/employees.sql docker-entrypoint-initdb.d/3_employees.sql && \
    sed -i 's|source load_departments.dump |source /test_db/load_departments.dump |g' /docker-entrypoint-initdb.d/3_employees.sql && \
    sed -i 's|source load_employees.dump |source /test_db/load_employees.dump |g' /docker-entrypoint-initdb.d/3_employees.sql && \
    sed -i 's|source load_dept_emp.dump |source /test_db/load_dept_emp.dump |g' /docker-entrypoint-initdb.d/3_employees.sql && \
    sed -i 's|source load_dept_manager.dump |source /test_db/load_dept_manager.dump |g' /docker-entrypoint-initdb.d/3_employees.sql && \
    sed -i 's|source load_titles.dump |source /test_db/load_titles.dump |g' /docker-entrypoint-initdb.d/3_employees.sql && \
    sed -i 's|source load_salaries1.dump |source /test_db/load_salaries1.dump |g' /docker-entrypoint-initdb.d/3_employees.sql && \
    sed -i 's|source load_salaries2.dump |source /test_db/load_salaries2.dump |g' /docker-entrypoint-initdb.d/3_employees.sql && \
    sed -i 's|source load_salaries3.dump |source /test_db/load_salaries3.dump |g' /docker-entrypoint-initdb.d/3_employees.sql && \
    sed -i 's|source show_elapsed.sql |source /test_db/show_elapsed.sql |g' /docker-entrypoint-initdb.d/3_employees.sql

# CMD [ "/entrypoint.sh" ]