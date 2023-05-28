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
env_logger = "0.10.0"
simple_logger = "4.1.0"
```

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

### `fn set_logger(logger: &'static dyn Log) -> Result<(), SetLoggerError>`

このメソッドを利用することで、アプリケーション内でグローバルに宣言されているロガーを設定することが可能であり、このメソッドを呼び出して初めてログの出力が可能となる。

[set_logger | log crate](https://docs.rs/log/latest/log/fn.set_logger.html)

このメソッドを呼び出さない場合には、マクロを実行した時には `NopLogger` という空の実装が用意されているオブジェクトのメソッドが実行される。

`info!` マクロを呼び出した時に、内部では `__private_api_log` 関数を呼び出しており、この中の `logger` 関数内部でロガーの初期化が実行されたかどうかを判定している。

```rs
pub fn __private_api_log(
    args: fmt::Arguments,
    level: Level,
    &(target, module_path, file, line): &(&str, &'static str, &'static str, u32),
    kvs: Option<&[(&str, &str)]>,
) {
    if kvs.is_some() {
        panic!(
            "key-value support is experimental and must be enabled using the `kv_unstable` feature"
        )
    }

    // この logger 関数内部でどのログ実装を使用するのかを判断する
    logger().log(
        &Record::builder()
            .args(args)
            .level(level)
            .target(target)
            .module_path_static(Some(module_path))
            .file_static(Some(file))
            .line(Some(line))
            .build(),
    );
}
```

[https://github.com/rust-lang/log/blob/f4c21c1b2dc958799eb6b3e8e713d1133862238a/src/lib.rs#L1468-L1490](https://github.com/rust-lang/log/blob/f4c21c1b2dc958799eb6b3e8e713d1133862238a/src/lib.rs#L1468-L1490)

実際に `logger` 関数の内容を確認すると以下のように `AtomicUsize` で管理している状態を取得し、初期化されたかどうかを判定させた後に実際に利用するロガーの判断を行なっている。

```rs
// ロガーの設定状態を管理する変数
static STATE: AtomicUsize = AtomicUsize::new(0);
const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

// グローバルに宣言されたロガーへのポイントを保持する
// AtomicUsizeで宣言された STATE により初期化されたかどうかを判定している
static mut LOGGER: &dyn Log = &NopLogger;

// ...

pub fn logger() -> &'static dyn Log {
    if STATE.load(Ordering::SeqCst) != INITIALIZED {
        static NOP: NopLogger = NopLogger;
        &NOP
    } else {
        unsafe { LOGGER }
    }
}
```

[https://github.com/rust-lang/log/blob/f4c21c1b2dc958799eb6b3e8e713d1133862238a/src/lib.rs#LL1348C1-L1350C2](https://github.com/rust-lang/log/blob/f4c21c1b2dc958799eb6b3e8e713d1133862238a/src/lib.rs#LL1348C1-L1350C2)

`AtomicUsize` はマルチスレッド環境でのデータ一貫性を担保するために設計された型であり、複数のスレッドからでも値を安全に操作することが可能である。

[AtomicUsize | std crate](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html)

ログ出力を行う際はマルチスレッド環境からでもロガーを呼び出す可能性はあるため、アトミックな操作でロガーが初期化されたかどうかを判定することで、安全にどのログを利用するかの判断を行なっている。

（ただ、正直なところアトミック操作やメモリ順序への理解度は怪しいので「Rust Atomics and Locks」を読みたい。）

ここで `AtomicUsize` を初期化状態の管理で使用しているのは、ロガーの定義が static なライフタイムを有している可変参照として定義されているからである。

可変参照であるためそのまま利用してしまうと、複数のスレッドからロガーの初期化が呼び出されてしまった場合、 `LOGGER` に対して同時アクセスを行いデータ競合が発生してしまう可能性がある。そのため `AtomicUsize` を利用して初期化が一度だけ安全に行われることを保証するためにこのような設計になっているのだと推察できる。

次に `set_logger` メソッドが内部でどのように初期化を行っているのかを確認する。

```rs
// この関数でグローバルに宣言されたロガーを受け取って、static mutな変数を変更する
pub fn set_logger(logger: &'static dyn Log) -> Result<(), SetLoggerError> {
    set_logger_inner(|| logger)
}

// この内部でロガーを変更するが、AtomicUsizeを利用することで安全に上書きするようにしている
fn set_logger_inner<F>(make_logger: F) -> Result<(), SetLoggerError>
where
    F: FnOnce() -> &'static dyn Log,
{
    let old_state = match STATE.compare_exchange(
        UNINITIALIZED, // 現在の値が第１引数と等しい場合に
        INITIALIZING,  // 現在の値を第２引数で指定した値に交換する
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(s) | Err(s) => s,
    };
    match old_state {
        UNINITIALIZED => {
            unsafe {
                LOGGER = make_logger();
            }
            STATE.store(INITIALIZED, Ordering::SeqCst);
            Ok(())
        }
        INITIALIZING => {
            while STATE.load(Ordering::SeqCst) == INITIALIZING {
                // TODO: replace with `hint::spin_loop` once MSRV is 1.49.0.
                #[allow(deprecated)]
                std::sync::atomic::spin_loop_hint();
            }
            Err(SetLoggerError(()))
        }
        _ => Err(SetLoggerError(())),
    }
}
```

[https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#L1352-L1382](https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#L1352-L1382)

`AtomicUsize` が提供する `compare_exchange` は、現在の値と第1引数で指定された値と比較して、同じ値の場合には第2引数で指定した値に置き換える。そして、関数の返り値に置き換え前の現在の値を返却する。

この状態の変更に関しては `Ordering::SeqCst` が指定されているため、必ず1度に1つのスレッドのみがアトミックに状態を `INITIALIZING` という初期化中であることを示す状態に変更することが可能となる。

もしもあるスレッドがログ設定を行なっている間に、他のスレッドがログ設定の関数を呼び出した場合には `old_state` に `INITIALIZING` が返却され、後続の処理でスピンループを行うことでそのスレッドでの初期化設定が完了するまで待機し、そのあとでエラーを返却している。

このような初期化処理を実現することで、グローバルにロガー設定が1度のみしか呼出されないことを保証している。

### `fn set_max_level(level: LevelFilter)`

`info!` マクロを呼び出せば、自動的にログレベル `Info` が設定された `Metadata` がログレコードに付与された状態となるが、これだけだと全てのログメッセージが表示されてしまうことになる。

そこで `log` クレートは `set_max_level` というログの出力を調整するための関数を用意している。

```rs
// ログレベルに関してもグローバルなアトミックの設定を有している
static MAX_LOG_LEVEL_FILTER: AtomicUsize = AtomicUsize::new(0);

// ...

pub fn set_max_level(level: LevelFilter) {
    MAX_LOG_LEVEL_FILTER.store(level as usize, Ordering::Relaxed);
}
```

[https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#LL1220C1-L1222C2](https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#LL1220C1-L1222C2)

ここで `Ordering::Relaxed` を設定して制約を緩めている背景は以下のISSUEで言及されている通り、現在設定されている最大のログレベルを取得する箇所が `Ordering::Relaxed` を設定しているためである。

[Confusing memory orderings for MAX_LOG_LEVEL_FILTER](https://github.com/rust-lang/log/issues/453)

他のライブラリでは、このメソッドは `Log` トレイトの実装を行なったロガーの初期化を行うメソッドの内部で利用されていることが多い。

例えば `simple_logger` の場合であれば、以下のようなロガーを生成する処理の中でログレベルを設定し、そのメソッド内部で `set_max_level` を呼び出している。

```rs
simple_logger::init_with_level(log::Level::Warn).unwrap();
```

ここで設定したログレベルを、どのように管理して、ログの出力判断を行う `enabled` でどのように使用しているのかは、それぞれライブラリの実装によって異なっている。

## 適用されている実装パターン

### Facade パターン

### Builder パターン

### Box::leakによるstatic参照パターン

## simple_logger

## env_logger

## fern

## tracing-log
