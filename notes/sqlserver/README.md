# SQL Serverを理解する

## 準備

ベースとしてMySQLが提供しているEmployeesデータを、任意のデータ量で作成できるようにしたい

```sql
CREATE TABLE employees (
    emp_no INT NOT NULL,
    birth_date DATE NOT NULL,
    first_name VARCHAR(14) NOT NULL,
    last_name VARCHAR(16) NOT NULL,
    gender CHAR(1) NOT NULL,
    hire_date DATE NOT NULL,
    PRIMARY KEY(emp_no),
);

CREATE TABLE departments (
    dept_no CHAR(4),
    dept_name VARCHAR(40),
    PRIMARY KEY(dept_no)
);

CREATE TABLE dept_emp (
    emp_no INT,
    dept_no CHAR(4),
    from_date DATE NOT NULL,
    to_date DATE,
    PRIMARY KEY(emp_no, dept_no),
    FOREIGN KEY(emp_no) REFERENCES employees(emp_no),
    FOREIGN KEY(dept_no) REFERENCES departments(dept_no)
);

CREATE TABLE dept_manager (
    dept_no CHAR(4),
    emp_no INT,
    from_date DATE NOT NULL,
    to_date DATE,
    PRIMARY KEY(dept_no, emp_no),
    FOREIGN KEY(emp_no) REFERENCES employees(emp_no),
    FOREIGN KEY(dept_no) REFERENCES departments(dept_no)
);

CREATE TABLE titles (
    emp_no INT,
    title VARCHAR(50) NOT NULL,
    from_date DATE NOT NULL,
    to_date DATE,
    PRIMARY KEY(emp_no, title, from_date),
    FOREIGN KEY(emp_no) REFERENCES employees(emp_no)
);

CREATE TABLE salaries (
    emp_no INT,
    salary INT,
    from_date DATE NOT NULL,
    to_date DATE,
    PRIMARY KEY(emp_no, from_date),
    FOREIGN KEY(emp_no) REFERENCES employees(emp_no)
);
```

## 参考資料

- [SQLServerインデックスを理解して検索を高速にする方法](https://anderson02.com/sqlserver-index/)
- [Use The Index Luke](https://use-the-index-luke.com/ja)
- [絵で見てわかるSQL Serverの仕組み](https://amzn.asia/d/gnU5Vk8)
