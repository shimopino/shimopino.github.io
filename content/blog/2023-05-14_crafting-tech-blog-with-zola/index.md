+++
title = "Zolaで始める技術ブログ"
draft = true

[taxonomies]
tags = ["Zola"]
+++

最近 Rust で遊んでいますが、その過程で得た知識を記録していくためにブログを作成することにしました。せっかくなので、ブログには Rust 製の静的サイトジェネレーターである [Zola](https://www.getzola.org/documentation/getting-started/overview/) を使います。

Zola は Jinja2 に似た Tera テンプレートエンジンを使用しており、これから見ていくように動的なテンプレートを通じて、静的な HTML ページに高速に変換することが可能です。

さらに、Zola は Github Pages との連携も簡単に行うことができ、Github Actions を通じて Zola のサイトをビルドし、その結果を Github Pages にデプロイすることが可能です。

今回は Zola を使って技術ブログを構築した手順を残していこうと思います。

## Zola のセットアップ

公式ドキュメントに記載されている `zola init myblog` コマンドを実行すれば、下記の構造のディレクトリ・ファイルが生成されます。

```bash
├── config.toml
├── content
├── sass
├── static
├── templates
└── themes
```

`config.toml` は Zola の設定ファイルであり、こちらにブログの URL であったり各種設定を行うことが可能です。

ディレクトリはそれぞれ以下の役割が設定されています。

- content
  - Markdown ファイルなどでサイトを構成する記事を管理するディレクトリ
  - 子ディレクトリを作成すればサイトの URL に反映される
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
  - 今回は 1 から作っていくので、このディレクトリは使用しない

初期状態のまま `zola build` を実行すると、下記の外観のサイトが構築されます。

![](assets/first-site.png)

[Overview | Zola](https://www.getzola.org/documentation/getting-started/directory-structure/)

これでブログを始める準備が整いました。

## Github Pages へのデプロイ

私はブログに限らず、ソフトウェアを構築する際には最終的な成果物を完成させてからデプロイするよりも、インクリメンタルに作成していくことが好みです。

まずは Web ページとして閲覧できる状態にするために、今回は Github Actions を使用して、Github Pages にデプロイすることを目指します。

Github Pages では `gh-pages, main, master` というブランチルートに `index.html` を配置してページを公開したり、リポジトリの `docs` ディレクトリから公開することも可能です。

Github Pages の URL は以下のパターンで決まります。

- 特定の名前のリポジトリ
  - リポジトリ名を `<username>.github.io` に設定する
  - これは例えば以下のようなサイトが該当する
    - https://github.com/Yelp/yelp.github.io
- それ以外のリポジトリ
  - `<username>.github.io/<repository>`

[Github Pages について](https://docs.github.com/ja/pages/getting-started-with-github-pages/about-github-pages)

Github Actions 経由でデプロイするには以下の 3 つのステップが必要となります。

1. 他のリポジトリ経由で公開する場合は、そのリポジトリから自身のリポジトリにプッシュするための権限を付与するためのパーソナルアクセストークンを生成する
2. Github Actions を用意する
3. リポジトリ設定の「Github Pages」の項目を設定する

今回はこのリポジトリからサイトを公開するため、PAT の準備はスキップします。

Github Actions は [zola-deploy-action](https://github.com/shalzz/zola-deploy-action) にサンプルが配置されているため、こちらを参考に構築します。

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
        # v0.17.2 では git config --global --add safe.directory '*' に失敗する時があった
        # 最新版の参照する形式に変更
        # https://github.com/shalzz/zola-deploy-action/issues/53#issuecomment-1409707948
        uses: shalzz/zola-deploy-action@master
        env:
          # https://docs.github.com/ja/actions/security-guides/automatic-token-authentication
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

Github Actions が実行されると、新しく `gh-pages` ブランチが作成され、そこに `zola build` によって生成された静的ファイルが配置されます。

![](assets/github-pages-visit.png)

これでローカルで確認した時と、同じ内容のサイトを構築することができました。

![](assets/github-pages-first-deploy.png)

[Github Pages | Deployment | Zola](https://www.getzola.org/documentation/deployment/github-pages/)
