+++
title = "logクレートが提供する柔軟性の仕組みを探る"
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
> Facade パターンとのつながり
> 自身でカスタムロガーを作成する
> 他の log トレイト実装のサンプル

## log クレートを利用した Rust での logging

log クレートは、それ自体はロギングの実装を提供しておらず、Rust で標準的なロギングを行うための API となるトレイトを提供している。そのため log クレートを利用してロギングを行う際には、実際の実装を提供するクレートと組み合わせる必要がある。

実装を提供するクレートには、例えば以下のものがある。

- [`env_logger`](https://docs.rs/env_logger/*/env_logger/)
- [`fern`](https://docs.rs/fern/*/fern/)
- [`tracing-log`](https://docs.rs/tracing-log/latest/tracing_log/)

環境変数を利用して設定を行うことが可能な `env_logger` では、以下のようなコードを記述するだけでログを出力することが可能である。

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

これだけだと内部でどのような処理を実現しているのかを推察することが難しいため、公式ドキュメントに記載されている自作ロガーのコードも確認する。

## 自作ロガーの実装を確認する

公式ドキュメントのサンプルでは `Log` トレイトの実装として以下がが提供されている。

```rs
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        println!("{:?}", metadata);
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        println!("{:?}", record);
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
```

そしてこの実装を呼び出す時には以下のように `set_logger` 関数を呼び出してグローバルに適用するロガーを登録し、ログレベルを設定して出力されるログを制御するようにしている。

```rs
static LOGGER: SimpleLogger = SimpleLogger;

fn main() {
    // env_logger::init();
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Warn));

    log::trace!("trace");
    log::debug!("debug");
    log::info!("info");
    log::warn!("warn");
    log::error!("error");
}
```

この場合であれば最大のログレベルに `Info` を設定しているため、`debug!` マクロや `trace!` マクロはメッセージを出力されないようになっている。

これからは `log` クレートが提供している下記の機能の詳細を見ていく。

- `Log` トレイト
- `set_logger`
- `set_max_level`

## Log トレイト

ログの実装を行うためには `log` クレートが提供している `Log` トレイトを実装することで、各マクロを実行したときのログ出力の挙動を制御する必要がある。

ここで `Log` トレイトの定義を確認すると、以下になっている。

```rs
pub trait Log: Sync + Send {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool;
    fn log(&self, record: &Record<'_>);
    fn flush(&self);
}
```

[Log trait | log crate](https://github.com/rust-lang/log/blob/502bdb7c63ffcbad4fe6921b46d582074e49fd0a/src/lib.rs#L1124C1-L1150)

この定義を 1 つ 1 つ見ていく。

### `pub trait Log: Sync + Send`

まずは `Log` トレイトのトレイト境界に設定されているマーカートレイトである `Send` トレイトと `Sync` トレイトを振り返る。

- `Send` トレイト
  - 実装した型の所有権をスレッド間で転送できることを表す
- `Sync` トレイト
  - 複数のスレッドから参照されても安全であることを表す

`Log` トレイトを実装する全ての型は、スレッド間で安全に転送でき、スレッド間で安全に参照を共有することを保証する必要がある。

例えばマルチスレッドでリクエストを処理するような Web サーバーの利用を考えると、各スレッドからは `Log` トレイトを実装したオブジェクトにアクセスできる必要がある。 `Sync` トレイトが実装されていれば、複数のスレッドから同時に安全にアクセスできることが保証される。

### `fn enabled(&self, metadata: &Metadata<'_>) -> bool;`

このメソッドを実行することで、以下の構造体で定義されているメタデータを含むログメッセージを記録するかどうかを判定する。

```rs
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Metadata<'a> {
    level: Level,
    target: &'a str,
}
```

`level` にはそれぞれログ出力時に呼び出したマクロに対するログレベルが設定されている。この値とグローバルに設定されたログレベルなどの比較を行い、ログを出力するのかどうかを判定することが可能である。

```rs
fn enabled(&self, metadata: &log::Metadata) -> bool {
    // 必ずInfoレベル以上のログを出力しないように設定している
    // 基本的にはグローバルで設定したものをキャプチャしてフィルタリングを行う
    metadata.level() <= Level::Info
}
```

`target` にはマクロを呼び出す際にオプションとして設定することが可能であり、例えばライブラリやアプリケーションの名前を設定することで、ログメッセージがどのモジュールから生成されたものを追跡できるようになっている。

例えば以下のようにエラーメッセージを出力する。

```ts
log::error!(target: "Global", "error");
```

この場合は設定したログレベルとターゲット情報をもとに `Metadata` が生成されていることがわかる。

```rs
Metadata { level: Error, target: "Global" }
```

まとめるとこのメソッドは、ログ出力時に呼び出したマクロのログレベルをキャプチャして、条件に基づいてログを出力するかどうかを決めることが可能なメソッドである。

### `fn log(&self, record: &log::Record)`

このメソッドを実行することで、ログをそもそも出力するかどうかの制御であったり、ログメッセージのフォーマットなどを制御することが可能である。

```rs
fn log(&self, record: &log::Record) {
    if self.enabled(record.metadata()) {
        println!("{} - {}", record.level(), record.args());
    }
}
```

このメソッドは、各マクロを呼び出した時に以下で定義されている `Record` を受け取り、ログマクロが実行されたときの情報を抽出することが可能となる。

```rs
#[derive(Clone, Debug)]
pub struct Record<'a> {
    metadata: Metadata<'a>,
    args: fmt::Arguments<'a>,
    module_path: Option<MaybeStaticStr<'a>>,
    file: Option<MaybeStaticStr<'a>>,
    line: Option<u32>,
    #[cfg(feature = "kv_unstable")]
    key_values: KeyValues<'a>,
}
```

ログマクロを実行したときに内部でこのレコードが生成され、指定したメッセージやマクロを呼び出した行数、実行したときのファイル名などが格納されている。

例えば以下のようにエラーメッセージを出力する。

```ts
log::error!(target: "Global", "error");
```

このときメタデータが格納されたレコードが生成され、Rustの標準ライブラリから提供されている `line!` マクロや `file!` マクロを呼び出した値で初期化を行っている。

```rs
Record { 
  metadata: Metadata { level: Error, target: "Global" },
  args: "error", 
  module_path: Some(Static("log")), 
  file: Some(Static("examples/log/main.rs")), 
  line: Some(31)
}
```

[macros | log crate](https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/macros.rs#L245-L267)

### `fn flush(&self);`

標準出力にメッセージを出すだけの場合にはあまり使うことはないかもしれないが、ログメッセージをファイルに出力したりする場合に利用する。

例えば `std::io::Write` トレイトでも `flush` メソッドは提供されており、以下のようにファイルを生成して書き込む内容を指定した後で、 `flush` を呼び出すことでバッファに書き込まれた内容をファイルに反映する。

```ts
fn main() -> std::io::Result<()> {
    let mut buffer = BufWriter::new(File::create("foo.txt")?);

    buffer.write_all(b"some bytes")?;
    buffer.flush()?;
    Ok(())
}
```

`Log` トレイトに限らず、 `flush` メソッドは上記のように、パフォーマンス向上のためにデータをメモリ上に保存して、一定の条件や任意のタイミングで永続的なストレージに書き出す時に利用される。

例えば `fern` クレートでは、出力先に応じてそれぞれ対応する `flush` メソッドを呼び出すことで、ファイルやチャンネルに対してメッセージを書き出す挙動を制御している。

[https://github.com/daboross/fern/blob/4f45ef9aac6c4d5929f100f756b5f4fea92794a6/src/log_impl.rs#L378-L407](https://github.com/daboross/fern/blob/4f45ef9aac6c4d5929f100f756b5f4fea92794a6/src/log_impl.rs#L378-L407)

## 適用されている実装パターン

### Facade パターン

### Builder パターン

## simple_logger

## env_logger

## fern

## tracing-log
