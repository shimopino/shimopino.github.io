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

## log クレート

log クレートは、それ自体はロギングの実装を提供しておらず、Rust で標準的なロギングを行うための API となるトレイトを提供している。そのため log クレートを利用してロギングを行う際には、実際の実装を提供するクレートと組み合わせる必要がある。

よく利用される実装は以下の通りである。

- [`env_logger`](https://docs.rs/env_logger/*/env_logger/)
- [`fern`](https://docs.rs/fern/*/fern/)
- [`tracing-log`](https://docs.rs/tracing-log/latest/tracing_log/)

本記事では、まず Log トレイトを自前で実装した後に、他のクレートがどのような実装を提供しているのかをみていく。

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

## env_logger

## fern

## tracing-logger
