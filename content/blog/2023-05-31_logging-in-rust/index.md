+++
title = "logクレートが提供する柔軟性の仕組みを探る"
description = "普段tracingクレートを使っているので、あたらめてlogクレートを調べる"
draft = false

[taxonomies]
tags = ["Rust", "Logging"]
+++

Rust でアプリケーションを作成する際に [`tracing`](https://docs.rs/tracing/latest/tracing/) クレートを利用する場合も多くありますが、プロジェクトの初期段階や簡単な POC であればよりシンプルな [`log`](https://docs.rs/log/latest/log/) クレートを利用する選択肢もあるかと思います。

本記事では `log` クレートの仕組みを追っていきながら、実装を提供している `simple_logger` クレートがどのように機能しているのか理解を深めていきます。

```toml
[dependencies]
log = "0.4.18"
simple_logger = "4.1.0"
```

## log クレートを利用した Rust での logging

`log` クレートは、それ自体はロギングの実装を提供しておらず、Rust で標準的なロギングを行うための API となるトレイトを提供しています。そのため `log` クレートを利用してロギングを行う際には、実際の実装を提供するクレートと組み合わせる必要があります。

以下は実装を提供しているクレートの一部です。

- [`env_logger`](https://docs.rs/env_logger/*/env_logger/)
- [`fern`](https://docs.rs/fern/*/fern/)
- [`tracing-log`](https://docs.rs/tracing-log/latest/tracing_log/)

環境変数を利用して設定を行うことが可能な `env_logger` では、以下のようなコードを記述するだけでログを出力することが可能です。

```rs
fn main() {
    env_logger::init();

    log::info!("hello");
}
```

後は環境変数を指定して実行するとログが出力されていることが確認できます。

```bash
$ RUST_LOG=info cargo run
...
[2023-05-27T09:50:55Z INFO  log] hello
```

これだけだと内部でどのような処理を実現しているのかを推察することが難しいため、公式ドキュメントに記載されている自作ロガーのコードも確認します。

## 自作ロガーの実装を確認する

公式ドキュメントのサンプルでは `Log` トレイトの実装として以下がが提供されています。

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

そしてこの実装を呼び出す時には以下のように `set_logger` 関数を呼び出してグローバルに適用するロガーを登録し、ログレベルを設定して出力されるログを制御するようにしています。

```rs
// SimpleLoggerはフィールドを持たないユニット構造体である
// 型の名前自体が唯一の値となるため、単に SimpleLogger と記述すればインスタンスを作成できる
// フィールドを有する場合には、そのフィールドを初期化する必要がある
static LOGGER: SimpleLogger = SimpleLogger;

fn main() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    log::trace!("trace");
    log::debug!("debug");
    log::info!("info");
    log::warn!("warn");
    log::error!("error");
}
```

この場合であれば最大のログレベルに `Info` を設定しているため、`debug!` マクロや `trace!` マクロはメッセージを出力されないようになっています。

これからは `log` クレートが提供している下記の機能の詳細を見ていきます。

- `Log` トレイト
- `set_logger` 関数
- `set_max_level` 関数

## Log トレイト

ログの実装を行うためには `log` クレートが提供している `Log` トレイトを実装することで、各マクロを実行したときのログ出力の挙動を制御する必要があります。

`Log` トレイトは以下のように定義されています。

```rs
pub trait Log: Sync + Send {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool;
    fn log(&self, record: &Record<'_>);
    fn flush(&self);
}
```

[Log trait | log crate](https://github.com/rust-lang/log/blob/502bdb7c63ffcbad4fe6921b46d582074e49fd0a/src/lib.rs#L1124C1-L1150)

この定義を 1 つ 1 つ見ていきます。

### `pub trait Log: Sync + Send`

まずは `Log` トレイトのトレイト境界に設定されているマーカートレイトである `Send` トレイトと `Sync` トレイトを振り返ります。

- `Send` トレイト
  - 実装した型の所有権をスレッド間で転送できることを表す
- `Sync` トレイト
  - 複数のスレッドから参照されても安全であることを表す

`Log` トレイトを実装する全ての型は、スレッド間で安全に転送でき、スレッド間で安全に参照を共有することを保証する必要があります。

例えばマルチスレッドでリクエストを処理するような Web サーバーの利用を考えると、各スレッドからは `Log` トレイトを実装したオブジェクトにアクセスできる必要があります。 `Sync` トレイトが実装されていれば、複数のスレッドから同時に安全にアクセスできることが保証されます。

実際には `Send` トレイトと `Send` トレイトから構成される型は自動的にこれらのトレイトを実装するので、手動で実装する必要はありません。

### `fn enabled(&self, metadata: &Metadata<'_>) -> bool;`

このメソッドを実行することで、以下の構造体で定義されているメタデータを含むログメッセージを記録するかどうかを判定します。

```rs
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Metadata<'a> {
    level: Level,
    target: &'a str,
}
```

`level` にはそれぞれログ出力時に呼び出したマクロに対するログレベルが設定されており、この値とグローバルに設定されたログレベルなどの比較を行い、ログを出力するのかどうかを判定することが可能です。

```rs
impl Log {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
      // 必ずInfoレベル以上のログを出力しないように設定している
      // 基本的にはグローバルで設定したものをキャプチャしてフィルタリングを行う
      metadata.level() <= Level::Info
  }
}
```

`target` にはマクロを呼び出す際にオプションとして設定することが可能であり、例えばライブラリやアプリケーションの名前を設定することで、ログメッセージがどのモジュールから生成されたものを追跡できるようになっています。

例えば以下のようにエラーメッセージを出力すると、

```ts
log::error!(target: "Global", "error");
```

この場合は設定したログレベルとターゲット情報をもとに `Metadata` が生成されていることがわかります。

```rs
Metadata { level: Error, target: "Global" }
```

まとめるとこのメソッドは、ログ出力時に呼び出したマクロのログレベルをキャプチャして、条件に基づいてログを出力するかどうかを決めることが可能なメソッドです。

またこのメソッドを呼び出すことが可能な `log_enabled!` マクロも用意されており、ログ出力時に重い計算が必要になる箇所ではこのマクロを利用することで出力する必要のない処理は実行しないように制御することが可能です。

```rs
if log_enabled!(log::Level::Debug) {
    log::info!("{}", expensive_call());
}
```

### `fn log(&self, record: &log::Record)`

このメソッドを実行することでログメッセージのフォーマットなどを制御することが可能であり、 `enabled` メソッドを呼び出してログの出力可否を細かく制御することも可能です。

```rs
fn log(&self, record: &log::Record) {
    if self.enabled(record.metadata()) {
        println!("{} - {}", record.level(), record.args());
    }
}
```

このメソッドは、各マクロを呼び出した時に以下で定義されている `Record` を受け取り、ログマクロが実行されたときの情報を抽出します。

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

ログマクロを実行したときに内部でこのレコードが生成され、指定したメッセージやマクロを呼び出した行数、実行したときのファイル名などが格納されています。

例えば以下のようにエラーメッセージを出力すると、

```ts
log::error!(target: "Global", "error");
```

このときメタデータが格納されたレコードが生成され、Rust の標準ライブラリから提供されている `line!` マクロや `file!` マクロを呼び出した値で初期化を行っています。（今回は検証のために作成したリポジトリ内で `examples` ディレクトリを作成して処理を実行させています。 ）

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

他のクレートではこのメソッドの中でタイムスタンプなどのフォーマットを行なっています。

### `fn flush(&self);`

標準出力にメッセージを出すだけの場合にはあまり使うことはないかもしれませんが、ログメッセージをファイルに出力したりする場合など利用します。

例えば `std::io::Write` トレイトでも `flush` メソッドは提供されており、以下のようにファイルを生成して書き込む内容を指定した後で、 `flush` を呼び出すことでバッファに書き込まれた内容をファイルに反映しています。

```ts
fn main() -> std::io::Result<()> {
    let mut buffer = BufWriter::new(File::create("foo.txt")?);

    buffer.write_all(b"some bytes")?;
    buffer.flush()?;
    Ok(())
}
```

`Log` トレイトに限らず、 `flush` メソッドは上記のように、パフォーマンス向上のためにデータをメモリ上に保存して、一定の条件や任意のタイミングで永続的なストレージに書き出す時などで利用されています。

他のクレートを例にとると、 `fern` クレートでは、出力先に応じてそれぞれ対応する `flush` メソッドを呼び出すことで、ファイルやチャンネルに対してメッセージを書き出す挙動を制御しています。

[https://github.com/daboross/fern/blob/4f45ef9aac6c4d5929f100f756b5f4fea92794a6/src/log_impl.rs#L378-L407](https://github.com/daboross/fern/blob/4f45ef9aac6c4d5929f100f756b5f4fea92794a6/src/log_impl.rs#L378-L407)

## 各種関数

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

`AtomicUsize` が提供する `compare_exchange` は、現在の値と第 1 引数で指定された値と比較して、同じ値の場合には第 2 引数で指定した値に置き換える。そして、関数の返り値に置き換え前の現在の値を返却する。

この状態の変更に関しては `Ordering::SeqCst` が指定されているため、必ず 1 度に 1 つのスレッドのみがアトミックに状態を `INITIALIZING` という初期化中であることを示す状態に変更することが可能となる。

もしもあるスレッドがログ設定を行なっている間に、他のスレッドがログ設定の関数を呼び出した場合には `old_state` に `INITIALIZING` が返却され、後続の処理でスピンループを行うことでそのスレッドでの初期化設定が完了するまで待機し、そのあとでエラーを返却している。

このような初期化処理を実現することで、グローバルにロガー設定が 1 度のみしか呼出されないことを保証している。

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

ここで `Ordering::Relaxed` を設定して制約を緩めている背景は以下の ISSUE で言及されている通り、現在設定されている最大のログレベルを取得する箇所が `Ordering::Relaxed` を設定しているためである。

[Confusing memory orderings for MAX_LOG_LEVEL_FILTER](https://github.com/rust-lang/log/issues/453)

他のライブラリでは、このメソッドは `Log` トレイトの実装を行なったロガーの初期化を行うメソッドの内部で利用されていることが多い。

例えば `simple_logger` の場合であれば、以下のようなロガーを生成する処理の中でログレベルを設定し、そのメソッド内部で `set_max_level` を呼び出している。

```rs
simple_logger::init_with_level(log::Level::Warn).unwrap();
```

ここで設定したログレベルを、どのように管理して、ログの出力判断を行う `enabled` でどのように使用しているのかは、それぞれライブラリの実装によって異なっている。

### `fn set_boxed_logger(logger: Box<dyn Log>) -> Result<(), SetLoggerError>`

`set_logger` 関数では `&'static dyn Log` 型を引数に取る都合上、 `Log` トレイトを実装したロガーは、プログラムの実行全体にわたって有効なものでないといけない。

そのため公式ドキュメントのサンプルでは、初期化を行う際に `static` でロガーを宣言するようにしていた。

```rs
struct SimpleLogger;
impl log::Log for SimpleLogger {
    // ...
}

static LOGGER: SimpleLogger = SimpleLogger;

fn main() {
    log::set_logger(&LOGGER).unwrap();
}
```

このように記述できるのは `SimpleLogger` がフィールドを持たないユニット構造体であり、その型の名前自体が唯一の値となるため `SimpleLogger` とだけ定義すればインスタンスを作成できるからである。

しかし、他のライブラリのようにロガーに対して各種設定を制御するためにフィールドを追加すると、他の方法でロガーを初期化して `static` な参照を取得する必要がある。

そのような場合に利用できるのは `set_boxed_logger` 関数である。

[set_boxed_logger | log crate](https://docs.rs/log/latest/log/fn.set_boxed_logger.html)

これは内部的には `set_logger_inner` 関数を呼び出しているだけではあるが、関数の引数が明確に異なっている。

```rs
pub fn set_boxed_logger(logger: Box<dyn Log>) -> Result<(), SetLoggerError> {
    set_logger_inner(|| Box::leak(logger))
}
```

ここで使用している `Box::leak` メソッドは、`Box` を使用してヒープ上に確保されたメモリを明示的にリークさせることで、そのメモリをプログラム終了時まで保持させることのできるメソッドである。

[Box::leak](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak)

このメソッドを実行することで `logger` をプログラム終了までヒープ上に保持させるようにし、その結果このメソッドから返却されるものは `&'static mut Log` の参照となり、エラーが発生することなくコンパイルすることが可能となる。

この `set_boxed_logger` を利用することで、 `static` な値で初期化することなく、以下のようにスコープ内で生成されたロガーをグローバルな変数として登録することが可能となる。

```rs
// simple_loggerの例
fn main() {
    // Box::leakを活用することで関数内で生成したロガーを static に登録できる
    SimpleLogger::new().init().unwrap();

    log::warn!("This is an example message.");
}
```

### `fn set_max_level(level: LevelFilter)`

`log` クレートではグローバルに最大のログレベルを設定することのできる関数 `set_max_logger` も提供している。

[set_max_logger | log crate](https://docs.rs/log/latest/log/fn.set_max_level.html)

このメソッドの役割は重要であり、この関数を通して設定されたログレベルを `info!` などの各種マクロで参照し、実際にログ出力を行うかどうかを判断している。

```rs
// log!(target: "my_target", Level::Info, "a {} event", "log");
(target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
    let lvl = $lvl;
    if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
        $crate::__private_api_log(
            __log_format_args!($($arg)+),
            lvl,
            &($target, __log_module_path!(), __log_file!(), __log_line!()),
            $crate::__private_api::Option::None,
        );
    }
});
```

[https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/macros.rs#L45-L56](https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/macros.rs#L45-L56)

この処理の中では以下の 2 つのログレベルを参照している。

- `STATIC_MAX_LEVEL`
  - フィーチャーフラグレベルで制御された最大のログレベル
  - リリースビルド時に出力したいログを制御するときに利用する
  - デフォルトでは `LevelFilter::Trace` が設定されている
  - [https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#L1586](https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#L1586)
- `max_level()`
  - プログラム側で設定する最大のログレベル
  - `set_max_level` 関数を通して制御する
  - デフォルトでは `LevelFilter::Off` が設定されている（つまり、何もログ出力しない）
  - [https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#L408](https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#L408)

`log` クレートでは、ログレベルとして以下の `Enum` を定義しており、各マクロに対応するログレベルと、全てのログを出力しないレベルに設定された `Off` のログレベルが定義されており、このログレベルが初期値として設定されている。

```rs
static MAX_LOG_LEVEL_FILTER: AtomicUsize = AtomicUsize::new(0);

pub enum LevelFilter {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}
```

[LevelFilter | log crate](https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#LL552C1-L567C2)

つまり明示的にこのログレベルを変更しなければ、デフォルトでは全てのログ出力は抑制されるようになっている。

```rs
// https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#LL1265C1-L1273C2
pub fn max_level() -> LevelFilter {
    unsafe { mem::transmute(MAX_LOG_LEVEL_FILTER.load(Ordering::Relaxed)) }
}

// https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#LL1220C1-L1222C2
pub fn set_max_level(level: LevelFilter) {
    MAX_LOG_LEVEL_FILTER.store(level as usize, Ordering::Relaxed);
}
```

`std::mem::transmute` は非常に危険な関数ではあるが、ある型から別の型へのビット単位の移動を意味しており、引数で指定した値から返り値で指定した型に対してビットをコピーする。

`log` クレートの場合ではマルチスレッドでログレベルの変更を管理するために `AtomicUsize` を利用しているため、ログレベルを定義している `LevelFilter` と `usize` で型が異なっている。関数のインターフェースレベルでは `LevelFilter` のみを表に出しているため、 `LevelFilter` をアトミックに更新するための裏技的なやり方である。

`match` 式などを利用してより安全に型変換を行う方法もあるが、どの値にも該当しない `exhaustive patterns` をどのように取り扱うのか、であったり単純なビット移動である `transmute` の方がパフォーマンスが良い、という理由で現状のコードになっている可能性はある。

[トランスミュート transmute](https://doc.rust-jp.rs/rust-nomicon-ja/transmutes.html)

## 全体像

![](./assets/log-overview.drawio.svg)

## log トレイトの実装を提供しているクレート

ここからは各種クレートがどのように `Log` トレイを実装しているのかを見ていく。

- [simple_logger](https://docs.rs/simple_logger)
- [env_logger](https://docs.rs/env_logger/)
- [fern](https://docs.rs/fern)
- [tracing_log](https://docs.rs/tracing-log/latest/tracing_log/)

よく利用されているであろうこれらのクレートを対象にする。

## simple_logger

[`simple_logger`](https://docs.rs/simple_logger) はロガーの設定や出力メッセージがとてもシンプルで使いやすいクレートであり、本体のコードも `lib.rs` のみで構成されているため `Log` トレイトの実装例確認の最初の一歩に適しています。

公式から提供されている Getting Started なコードを確認すると、今まで説明してきた `set_boxed_logger` によるグローバルなロガーの宣言や `set_max_level` での最大ログレベルの設定を行なっていると予想できる。

```rs
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();

    log::warn!("This is an example message.");
}
```

これで以下のようにログメッセージが表示される。

```bash
2023-05-30T11:49:38.789Z WARN  [simple] This is an example message.
```

このクレートでは関連関数を使用していることからわかるように `SimpleLogger` のインスタンス生成と設定適用の関数をそれぞれ役割に分けて分離させている。

```rs
impl SimpleLogger {
    #[must_use = "You must call init() to begin logging"]
    pub fn new() -> SimpleLogger {
        SimpleLogger {
            default_level: LevelFilter::Trace,
            module_levels: Vec::new(),

            // 各フィーチャーフラグで有効化させるプロパティ
        }
    }
}
```

[https://github.com/borntyping/rust-simple_logger/blob/3a78bcf7ab4f4b594c0b55290afe42a50b6a295f/src/lib.rs#LL105C1-L123C6](https://github.com/borntyping/rust-simple_logger/blob/3a78bcf7ab4f4b594c0b55290afe42a50b6a295f/src/lib.rs#LL105C1-L123C6)

ここでは `#[must_use]` 属性を利用することで以下のようにロガー設定を行うための `init` 関数を呼び出していない場合には警告を発するようになっている。

```rs
fn main() {
    // warningが発生する
    SimpleLogger::new();
}
```

```rs
pub fn init(mut self) -> Result<(), SetLoggerError> {
    // ...

    self.module_levels
        .sort_by_key(|(name, _level)| name.len().wrapping_neg());
    let max_level = self.module_levels.iter().map(|(_name, level)| level).copied().max();
    let max_level = max_level
        .map(|lvl| lvl.max(self.default_level))
        .unwrap_or(self.default_level);
    log::set_max_level(max_level);
    log::set_boxed_logger(Box::new(self))?;
    Ok(())
}
```

[https://github.com/borntyping/rust-simple_logger/blob/3a78bcf7ab4f4b594c0b55290afe42a50b6a295f/src/lib.rs#LL347C1-L363C6](https://github.com/borntyping/rust-simple_logger/blob/3a78bcf7ab4f4b594c0b55290afe42a50b6a295f/src/lib.rs#LL347C1-L363C6)

この `init` 関数で最大のログレベルの設定やロガーのグローバルな値として登録を行なっている。また最大のログレベルは `module_levels` を調整するか `default_level` を調整する 2 つの方法があることがわかり、それぞれ `SimpleLogger` が提供している `with_module_level` 関数や `with_level` 関数を通して制御することが可能である。

`SimpleLogger` では `env_logger` の挙動を模倣させ環境変数からも最大のログレベルを設定することが可能である。

```rs
#[must_use = "You must call init() to begin logging"]
pub fn env(mut self) -> SimpleLogger {
    self.default_level = std::env::var("RUST_LOG")
        .ok()
        .as_deref()
        .map(log::LevelFilter::from_str)
        .and_then(Result::ok)
        .unwrap_or(self.default_level);

    self
}
```

[https://github.com/borntyping/rust-simple_logger/blob/3a78bcf7ab4f4b594c0b55290afe42a50b6a295f/src/lib.rs#LL157C1-L167C6](https://github.com/borntyping/rust-simple_logger/blob/3a78bcf7ab4f4b594c0b55290afe42a50b6a295f/src/lib.rs#LL157C1-L167C6)

こうした環境変数からの読み取りを行うメソッドが提供されているため、このメソッドを初期化の際に利用すれば、 `RUST_LOG=info cargo run` という形式で最大のログレベルを設定することも可能である。 `dotenvy` などと組み合わせれば、アプリケーションを動作させる環境ごとに異なるログレベルを設定することも容易である。

`log` クレートが提供している `LevelFilter` は `FromStr` トレイトを実装しているため、環境変数から取得した文字列と事前に定義されたログレベルの文字列との比較を行うことで、対象の型への変換を行なっている。

```rs
static LOG_LEVEL_NAMES: [&str; 6] = ["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

impl FromStr for LevelFilter {
    type Err = ParseLevelError;
    fn from_str(level: &str) -> Result<LevelFilter, Self::Err> {
        ok_or(
            LOG_LEVEL_NAMES
                .iter()
                .position(|&name| name.eq_ignore_ascii_case(level))
                .map(|p| LevelFilter::from_usize(p).unwrap()),
            ParseLevelError(()),
        )
    }
}
```

[https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#LL583C1-L594C2](https://github.com/rust-lang/log/blob/304eef7d30526575155efbdf1056f92c5920238c/src/lib.rs#LL583C1-L594C2)

これらの設定を簡易的に行うための専用の関数も用意されている。

```rs
pub fn init_with_env() -> Result<(), SetLoggerError> {
    SimpleLogger::new().env().init()
}
```

[https://github.com/borntyping/rust-simple_logger/blob/3a78bcf7ab4f4b594c0b55290afe42a50b6a295f/src/lib.rs#LL542C1-L544C2](https://github.com/borntyping/rust-simple_logger/blob/3a78bcf7ab4f4b594c0b55290afe42a50b6a295f/src/lib.rs#LL542C1-L544C2)

## 適用されている実装パターン

### Facade パターン

### Builder パターン

### Box::leak による static 参照パターン

### once_cell

### #[must_use]

- https://doc.rust-lang.org/std/hint/fn.must_use.html
- https://tech-blog.optim.co.jp/entry/2021/12/03/080000

### FromStr
