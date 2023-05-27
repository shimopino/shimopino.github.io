# データベースにより馴染むための項目

ChatGPTを駆使して、データベースに関する知見をより深めるための習得項目をまとめていく。可能であれば、下記項目に対して実際にPostgreSQLやMySQLなどを利用して、手を動かしながら理解を深めていくことを目指す。

## 項目まとめ

- インデックスなどのパフォーマンス最適化
  - インデックスの種類（B-Tree, Bitmap, Hash, etc.)
  - それらの最適な使用状況。
  - インデックスの作成方法
  - クエリのパフォーマンスを評価するためのツールと手法（EXPLAIN PLANなど）。
  - 適切なカラムの選択、インデックスのメンテナンス
  - データベース設計と正規化におけるパフォーマンス考慮。
- トランザクションやロックなどの挙動
  - トランザクションのACID特性とその重要性。
  - ロックの種類（共有ロック、排他ロック）とデッドロックの防止。
  - 同時実行制御（Concurrency Control）とその戦略（Optimistic, Pessimistic）。
  - トランザクションの制御方法
  - ロックの種類
  - ロックの制御方法
- 実際にデータベースを運用するための知見
  - バックアップとリカバリ戦略
  - データベースのセキュリティ、認証、認可。
  - データベースの監視とトラブルシューティング。
- RDBやNoSQLなどのデータベースの違いによる設計の違い
  - リレーショナルデータベースとNoSQLの違いとそれぞれの適切な使用状況。
  - データベースの種類（Key-Value, Document, Columnar, Graph etc.）とそれらの設計と使用。
  - CAP理論とデータ一貫性モデル（Strong, Eventual etc.）。
- データモデリングと設計
  - ERモデリング、データベース設計のベストプラクティス。
  - データ整合性の保証（主キー、外部キー、チェック制約）。
- データベースのテストとデバッグ: データベースの問題を解決するためのツールとテクニックについて説明すると良いでしょう。

## 準備

## 複雑なデータ分析のためのクエリ

### 演習　映画のレンタル回数をカテゴリ別に集計し、レンタル回数が1000回を超えているカテゴリ名とレンタル回数を表示する

<details>
<summary>回答</summary>

```sql
SELECT c.name AS category_name, COUNT(r.rental_id) AS rental_count
FROM category c
JOIN film_category fc ON c.category_id = fc.category_id
JOIN inventory i ON fc.film_id = i.film_id
JOIN rental r ON i.inventory_id = r.inventory_id
GROUP BY c.name
HAVING COUNT(r.rental_id) >= 5000;
```
 
</details>

### 演習　各顧客が支払った料金の合計を計算し、合計が200ドル以上の顧客の名前と支払った料金の合計を表示する

<details>
<summary>回答</summary>

```sql
SELECT c.first_name, c.last_name, SUM(p.amount) AS total_payment
FROM customer c
JOIN payment p ON c.customer_id = p.customer_id
GROUP BY c.customer_id, c.first_name, c.last_name
HAVING SUM(p.amount) >= 200;
```
 
</details>

## 演習　各カテゴリの映画の平均レンタル料金が、全体の平均レンタル料金より高いカテゴリを探す

<details>
<summary>回答</summary>

```sql
SELECT c.name AS category_name, AVG(f.rental_rate) AS average_rental_rate
FROM category c
JOIN film_category fc ON c.category_id = fc.category_id
JOIN film f ON fc.film_id = f.film_id
GROUP BY c.name
HAVING AVG(f.rental_rate) > (
  SELECT AVG(rental_rate) 
  FROM film
);
```
 
</details>

## 演習　レンタル回数が最も多い顧客を探す

<details>
<summary>回答</summary>

サブクエリを利用したやり方

```sql
SELECT c.first_name, c.last_name, COUNT(r.rental_id) AS rental_count
FROM customer c
JOIN rental r ON c.customer_id = r.customer_id
GROUP BY c.customer_id, c.first_name, c.last_name
HAVING COUNT(r.rental_id) = (
  SELECT MAX(rental_count) 
  FROM (
    SELECT COUNT(rental_id) AS rental_count
    FROM rental
    GROUP BY customer_id
  ) AS subquery
);
```
 
共通テーブル式（CTE）を利用したやり方

```sql
WITH customer_rentals AS (
  SELECT c.customer_id, c.first_name, c.last_name, COUNT(r.rental_id) AS rental_count
  FROM customer c
  JOIN rental r ON c.customer_id = r.customer_id
  GROUP BY c.customer_id, c.first_name, c.last_name
),
max_rentals AS (
  SELECT MAX(rental_count) AS max_rental_count
  FROM customer_rentals
)
SELECT cr.first_name, cr.last_name, cr.rental_count
FROM customer_rentals cr, max_rentals mr
WHERE cr.rental_count = mr.max_rental_count;
```

</details>

### 演習1

サブクエリ、結合、ウィンドウ関数、CTE (Common Table Expressions) 、グループ化クエリ
  - ビュー
  - マクロ
  - SQLパターン（例：ランキング、パーティション、パーセンタイル等）の理解と適用。

集計関数: COUNT, SUM, AVG, MIN, MAX などの基本的な集計関数を超えて、GROUP_CONCAT、ARRAY_AGG などのより高度な集計関数について学習します。

CASE文: CASE 文を使用して、結果セットの行に対して条件付きのロジックを適用する方法。

NULLの扱い: SQLではNULLは特殊な意味を持つため、NULL値の振る舞いとそれを扱うための関数（IS NULL, IS NOT NULL, COALESCE, IFNULL, NULLIF等）を理解することが重要です。

データ型と変換: SQLの各種データ型（数値、文字列、日付/時間など）と、それらの間での型変換についての理解。

正規表現: データの検索や置換に正規表現を使用する方法。

ストアドプロシージャと関数: これらはSQLのスクリプトをデータベースに保存し再利用するための方法です。繰り返し行われる複雑な操作を自動化するのに役立ちます。

分析関数: LEAD, LAG, FIRST_VALUE, LAST_VALUE などの関数を使用して、結果セット内の行と関連する他の行を比較する方法。

データの整形: PIVOT（行を列に変換）とUNPIVOT（列を行に変換）を使ったデータの整形。


