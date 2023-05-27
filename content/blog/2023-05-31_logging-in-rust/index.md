+++
title = "Rustにおけるロギング（トレーシングではないよ）"
description = "普段tracingクレートを使っているので、あたらめてlogクレートを調べる"
draft = false

[taxonomies]
tags = ["Rust", "Monitoring"]
+++

Rust でアプリケーションを作成する際に [`tracing`](https://docs.rs/tracing/latest/tracing/) クレートを利用する場合も多くありますが、プロジェクトの初期段階や簡単な POC であればよりシンプルな [`log`](https://docs.rs/log/latest/log/) クレートを利用する選択肢もあるかと思います。

本記事では `log` クレートの仕組みを追っていきながら、将来的に `tracing` クレートに移行することを視野に入れてどのようにロギングの実装をしていけば良いのかを調査していく。

```toml
[dependencies]
log = "0.4.17"
```

> 後で消す
> 記事の流れ
> 一般的な log クレートの使い方をまず提示して、最終結果がどうなるのかを抑える
> 簡単な log クレートの仕組みを図解を用いて提示する
> より詳細な実装に入る（各構造体、マクロ内部）
> Facadeパターンとのつながり
> 自身でカスタムロガーを作成する
> 他の log トレイト実装のサンプル

## logクレートを利用したRustでのlogging

log クレートは、それ自体はロギングの実装を提供しておらず、Rust で標準的なロギングを行うための API となるトレイトを提供している。そのため log クレートを利用してロギングを行う際には、実際の実装を提供するクレートと組み合わせる必要がある。

実装を提供するクレートには、例えば以下のものがある。

- [`env_logger`](https://docs.rs/env_logger/*/env_logger/)
- [`fern`](https://docs.rs/fern/*/fern/)
- [`tracing-log`](https://docs.rs/tracing-log/latest/tracing_log/)

実際に環境変数を利用して設定を行うことが可能な `env_logger` では、以下のようなコードを記述するだけでログを出力することが可能である。

```rs
fn main() {
    env_logger::init();

    log::info!("hello");
}
```

後は環境変数を指定して実行するとログが出力されていることが確認できる。

```bash
$ RUST_LOG=info cargo run
...
[2023-05-27T09:50:55Z INFO  log] hello
```

まずはこの実装を見ていきながら、log クレートではどのような処理や抽象化を行うことで、ログの実装を切り替えるようにしているのかを見ていく。

## Log トレイト

`Log` トレイトは以下のように定義されているが、この定義の1つ1つをみていく。

```rs
pub trait Log: Sync + Send {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool;
    fn log(&self, record: &Record<'_>);
    fn flush(&self);
}
```

[Log trait | log crate](https://github.com/rust-lang/log/blob/502bdb7c63ffcbad4fe6921b46d582074e49fd0a/src/lib.rs#L1124C1-L1150)

### `pub trait Log: Sync + Send`



## Facade パターン

### トレイトのスコープ

### 具体的な実装

## simple_logger

## env_logger

## fern

## tracing-logger
