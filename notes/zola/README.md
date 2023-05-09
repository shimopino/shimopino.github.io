# Zola 事始め

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
