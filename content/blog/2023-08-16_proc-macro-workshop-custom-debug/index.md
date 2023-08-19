+++
title = "proc-macro-workshop で Rust の手続き的マクロに入門する"
description = "proc-macro-workshop で Rust の手続き的マクロに入門する"
draft = true

[taxonomies]
tags = ["Rust", "macro"]
+++

## 進め方

まずは本記事では `#[derive]` マクロを使って Builder パターンの実装を進めていき、最終的には以下のような処理を実現できるようにしていきます。

```rust
use derive_debug::CustomDebug;

#[derive(CustomDebug)]
pub struct Field {
    name: String,
    #[debug = "0b{:08b}"]
    bitmask: u8,
}
```

- [Derive macro: derive(CustomDebug)](https://github.com/dtolnay/proc-macro-workshop/tree/master#derive-macro-derivecustomdebug)

このマクロを適用した構造体を標準出力に表示すると、以下のように表示される。

```bash
Field { name: "st0", bitmask: 0b00011100 }
```

課題を進めていく上で、以下のクレートを利用します。それぞれの細かい説明は課題を進めていく中で紹介します。

```toml
[dependencies]
proc-macro2 = "1.0.66"
quote = "1.0.32"
syn = { version = "2.0.28", features = ['extra-traits'] }
```

## 01-parse

- [ ] ほとんど1つ前の課題と変化なし
- [ ] ライフタイムへの言及はあっても良さそう

## 02-impl-debug

- [ ] std::fmt::Debug トレイトを実装する前に、このトレイトの軽い説明を追加