# Zola 事始め

- [Zola 事始め](#zola-事始め)
  - [ディレクトリ構造](#ディレクトリ構造)
    - [疑問](#疑問)
  - [Github Pages へのデプロイ](#github-pages-へのデプロイ)
    - [疑問点](#疑問点)
  - [最初のページ作成](#最初のページ作成)
    - [親のテンプレートファイルを作成](#親のテンプレートファイルを作成)
    - [子側のテンプレートファイルで上書きする](#子側のテンプレートファイルで上書きする)
    - [ブログセクションの作成](#ブログセクションの作成)
    - [ブログコンテンツの作成](#ブログコンテンツの作成)
  - [Front Matter](#front-matter)
    - [疑問点](#疑問点-1)

## ディレクトリ構造

公式ドキュメントの記載されている `zola init myblog` コマンドを実行すると、以下の構造のディレクトリ・ファイルが生成される。

```bash
├── config.toml
├── content
├── sass
├── static
├── templates
└── themes
```

`config.toml` は Zola の設定ファイルであり、こちらにブログの URL であったり各種設定を行う必要がある。

ディレクトリはそれぞれ以下の役割が設定されている。

- content
  - Markdown ファイルなどでサイトを構成する記事を管理するディレクトリであり、子ディレクトリを作成すればサイトの URL に反映される。
- sass
  - コンパイルされる Sass ファイルを配置する
  - Sass 以外のファイルは無視される
  - ディレクトリ構造は保持されるため `sass/something/site.scss` は `public/something/site.css` にコンパイルされる
- static
  - 任意の種類のファイルを配置する
  - このディレクトリ内に配置した構造は、そのまま出力ディレクトリにコピーされる
  - 静的ファイルが大きい場合を考慮して、設定ファイルに `hard_link_static = true` を指定すれば、コピーせずにハードリンクする
- templates
  - レンダリングする時に使用する `Tera` のテンプレートファイルを格納する
  - 構文に従って変数などを指定できる
- themes
  - テーマを利用すると、ここにテンプレートファイル一式が保存される
  - テーマを使用しない場合は空のままにしておく

初期状態のまま `zola build` を実行すると、以下のようなサイトが生成される。

![](assets/first-site.png)

- https://www.getzola.org/documentation/getting-started/directory-structure/

### 疑問

- CSS ファイルを配置した場合はどうなる？

## Github Pages へのデプロイ

ブログを始めるにあたり、最終的な成果物を完成させてからデプロイするよりも、インクリメンタルに作成していくことが個人的な好みなので、まずは Web ページとして閲覧できる状態にする。

今回は Github Actions を使用して、Github Pages にデプロイすることを目指す。

Github Pages では `gh-pages, main, master` というブランチルートに `index.html` を配置して生成ファイルを公開したり、リポジトリの `docs` ディレクトリから公開することも可能である。

Github Pages の URL は以下のパターンで決まる。

- 特定の名前のリポジトリ

  - リポジトリ名を `<username>.github.io` に設定する
  - これは例えば以下のようなサイトが該当する
    - https://github.com/Yelp/yelp.github.io

- それ以外のリポジトリ
  - `<username>.github.io/<repository>`

`zola` を使用する場合は、スタイルをサブモジュールとして含めるようにすればうまく動作するらしい

```bash
git submodule add https://github.com/getzola/after-dark.git themes/after-dark
```

- https://docs.github.com/ja/pages/getting-started-with-github-pages/about-github-pages

Github Actions 経由でデプロイするには以下の 3 つのステップが必要となる。

1. もしも他のリポジトリから公開する場合は、そのリポジトリから自身のリポジトリにプッシュするための権限を付与するためのパーソナルアクセストークンを生成する
2. Github Actions を用意する
3. リポジトリ設定の「Github Pages」の項目を設定する

今回はこのリポジトリからサイトを公開するため、PAT の準備はスキップする。

Github Actions は [zola-deploy-action](https://github.com/shalzz/zola-deploy-action) の公式サンプルにならって構築する。

```yml
name: Zola on Github Pages

on:
  push:
    branches:
      - main

jobs:
  build:
    name: Publish Site
    runs-on: ubuntu-latest
    steps:
      - name: Checkout main
        uses: actions/checkout@v3.0.0
      - name: Build and Deploy
        uses: shalzz/zola-deploy-action@v0.17.2
        env:
          # https://docs.github.com/ja/actions/security-guides/automatic-token-authentication
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

この Github Actions が実行されると、新しく `gh-pages` ブランチが作成され、そこに `zola build` によって生成された静的ファイルが配置される。

![](assets/github-pages-visit.png)

これでローカルで初期化した後でビルドした時と同じサイトを構築することができる。

![](assets/github-pages-first-deploy.png)

- https://www.getzola.org/documentation/deployment/github-pages/

### 疑問点

サブモジュールによるスタイル適用の仕組みは何か？

## 最初のページ作成

### 親のテンプレートファイルを作成

公式ページの手順に従ってサンプルページを作成していく。

`template` ディレクトリでは、　`Tera` の構文に従ったテンプレートファイルを定義することができ、ここで定義した HTML ファイルを元に様々なページを作成していく。

以下のように `template/base.html` を作成すれば、 `block` で定義した箇所を child として設定したファイルで上書きすることが可能となる。

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>MyBlog</title>
  </head>

  <body>
    <section class="section">
      <div class="container">{% block content %} {% endblock content %}</div>
    </section>
  </body>
</html>
```

```html
{% extends "base.html" %} {% block content %}
<h1 class="title">This is my blog made with Zola.</h1>
{% endblock content %}
```

- https://tera.netlify.app/docs/#base-template

### 子側のテンプレートファイルで上書きする

子側のテンプレートファイルでは、拡張対象のテンプレートで定義されているブロック `content` を上書きすることができる。

`template/index.html` を作成し、以下のように `content` を定義すれば親側の `content` を指定した要素で上書きすることができる。

```html
{% extends "base.html" %}

<!-- 以下に上書きするブロック content を記述していく -->

{% block content %}
<h1 class="title">This is my blog made with Zola.</h1>
{% endblock content %}
```

![](assets/first-home-page.png)

### ブログセクションの作成

`content` ディレクトリに Markdown ファイルで記事の内容を記述していく。

```bash
├── content
│   └── blog
│       └── _index.md
```

上記の構造でファイルを定義すれば、 `<path>/blog` の URL の設定を記述することが可能となる。

例えば TOML 形式で以下の設定を記述すれば、対象 URL で使用するテンプレートファイルであったり、個別の記事で使用するテンプレートやページのタイトル、記事のソート順を指定することができる。

```md
+++
title = "List of blog posts"
sort_by = "date"
template = "blog.html"
page_template = "blog-page.html"
+++
```

次にブログのトップページと個別の記事で利用するテンプレートファイルを準備する。

ここでは設定で記述した通りに `template/blog.html` や `template/blog-page.html` を定義していく。

```html
{% extends "base.html" %}

<!-- 以下で content をどのように上書きするのか記載していく -->

{% block content %}
<h1 class="title">{{ section.title }}</h1>
<ul>
  <!-- section （今回では blog） に配置されているコンテンツを一覧で取得する -->
  {% for page in section.pages %}
  <!-- pageオブジェクトで個別の設定を使用する -->
  <li><a href="{{ page.permalink | safe }}">{{ page.title }}</a></li>
  {% endfor %}
</ul>
{% endblock content %}
```

### ブログコンテンツの作成

`content/blog` ディレクトリ以下に個別の記事の内容を作成していく。

今回はブログセクションの設定で追加したように `title` と、セクションページで記事をソートするための `date` を設定する。

```md
+++
title = "My first post"
date = 2019-11-27
+++

This is my first blog post.
```

個別の記事に対して、今までと同じように `content` をどのように上書きしていくのかをテンプレートファイルで記述していく。そのときにコンテンツの内容には `page.content` でアクセすることができる。

```html
{% extends "base.html" %}

<!-- 以下に個別の記事の content を定義する -->

{% block content %}
<h1 class="title">{{ page.title }}</h1>
<p class="subtitle"><strong>{{ page.date }}</strong></p>
<!-- ここにMarkdownファイルで記述した内容が挿入される -->
{{ page.content | safe }}
<!--  -->
{% endblock content %}
```

同じようなコンテンツファイルを配置すれば、以下のようにセクションファイルと個別のページを作成することができている状態となる。

- セクション: `<url>/blog`

  ![](assets/first-section.png)

- ページ: `<url>/blog/first`

  ![](assets/first-contents.png)

## Front Matter

`content` 以下に配置する `_index.md` は、対象のセクションに表示するコンテンツやメタデータの設定を行うことができる。

例えば以下のような `_index.md` が存在していた場合、これは `base_url/blogs` 以下のセクションでのコンテンツやメタデータの設定を行うことが可能である。

```bash
└──content
    └── blogs
        ├── _index.md
        ├── entry1.md
        └── entry2.md
```

この設定は以下のように `+++` で囲まれたファイルの冒頭で宣言することができ、宣言した内容はテンプレートから `section.content` 変数で利用できるようになる。

以下に使う可能性がありそうなものだけを抽出する。

```toml
+++
# htmlの <title> と同じようにタイトルを設定可能
title = "Blog Title"

# 各種CLIで `--drafts` を付与したした時にのみ読み込むかどうか
# 下書きなら true にしてビルドされないようにすれば良さそう
draft = false

# コンテンツをどのようにソートするのか指定できる
# ブログOnlyならおおよそ投稿日時とかで良さそう
sort_by = "none"

# 明示的にセクションでどのテンプレートを使用するのか指定できる
# セクションごとにテンプレートを作成しておくのが良さそう
template = "section.html"

# セクションページも検索インデックスに含めるかどうか
in_search_index = true

# セクションのURLにアクセスされた場合のリダイレクト先を決定する
# 例えばセクションに直接アクセスされた時に 404 ページを表示したくない時などに使う
redirect_to =
+++

コンテンツを記述可能
```

記事をソートすることもでき、以下のディレクトリ構造出会った場合に、`_index.md` の設定に `sort_by = "date"` を設定し、各ページには `date = 2023-04-01` などと設定すればその順番でソートされる。

```bash
└──content
    └── blogs
        ├── _index.md
        ├── entry1.md
        ├── entry2.md
        └── entry3.md
```

テンプレート側には、以下のようにセクション配下のページ一覧を表示する時に、この順番で表示される。

```tera
{% for post in section.pages %}
  <h1><a href="{{ post.permalink }}">{{ post.title }}</a></h1>
{% endfor %}
```

```md
+++
title = "List of blog posts"
sort_by = "date"
template = "blog.html"
page_template = "blog-page.html"
+++
```

```html
{% extends "base.html" %} {% block content %}
<h1 class="title">{{ section.title }}</h1>
<ul>
  <!-- If you are using pagination, section.pages will be empty. You need to use the paginator object -->
  {% for page in section.pages %}
  <li><a href="{{ page.permalink | safe }}">{{ page.title }}</a></li>
  {% endfor %}
</ul>
{% endblock content %}
```

```md
+++
title = "My first post"
date = 2019-11-27
+++

This is my first blog post.
```

```html
{% extends "base.html" %} {% block content %}
<h1 class="title">{{ page.title }}</h1>
<p class="subtitle"><strong>{{ page.date }}</strong></p>
{{ page.content | safe }} {% endblock content %}
```

- https://www.getzola.org/documentation/content/section/#front-matter

### 疑問点

- draft の挙動認識が合っているの
