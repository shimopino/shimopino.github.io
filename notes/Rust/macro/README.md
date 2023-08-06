# Rust の手続き的マクロをマスターする

## はじめに

Rust でプログラミングをしていると、 `vec!` や `println!` のような、!で終わる特別な関数を目にすることがあります。これらは、Rust の「マクロ」と呼ばれる機能です。

マクロは、簡単に言うと「コードを生成するコード」のようなものです。これにより、繰り返しや特定のパターンのコードを簡単に、効率的に書くことができます。この記事では、手続きマクロを中心に、 [`proc-macro-workshop`](https://github.com/dtolnay/proc-macro-workshop) という資料を元に学び進めていきます。

## Rust のマクロについて

Rust のマクロには、宣言的マクロと手続き的マクロの 2 つの種類が存在します。

- 宣言的マクロ: これは `macro_rules!` 構文で定義され、 `vec!` や `println!` のようなものがこれに該当します。
- 手続き的マクロ: このマクロの種類には以下の 3 つが含まれます。
  - derive マクロ: `#[derive]` 属性を使ってコードの追加を指定するもの。
  - attribute マクロ: さまざまな要素に適用できるカスタム属性を定義するためのもの。
  - function マクロ: 関数のように、与えられた引数に基づいて動作するマクロ。

本記事では `proc-macro-workshop` を通じて、手続き的マクロの各種類とその記述方法について理解度を深めていきます。

まずは本記事では `#[derive]` マクロを使ってBuilderパターンの実装を進めていき、最終的には以下のような処理を実現できるようにしていきます。

```rust
use derive_builder::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
    #[builder(each = "arg")]
    args: Vec<String>,
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .arg("build".to_owned())
        .arg("--release".to_owned())
        .build()
        .unwrap();

    assert_eq!(command.executable, "cargo");
}
```

- [Derive macro: derive(Builder)](https://github.com/dtolnay/proc-macro-workshop/tree/master#derive-macro-derivebuilder)

## 疑問点

- [ ] trybuild とは何か？
- [ ] Cargo.toml における test とは何か？
- [ ] 外部クレートを利用せずに実装するには？
- [ ] parse_macro_input
- [ ] DeriveInput
- [ ] proc_macro_derive とその他のマクロとの違いは何か？
- [ ] 単純な `Command` を Builder マクロを適用した時の `DeriveInput` の具体的な Debug 内容（ `{:#?}` ）
- [ ] proc_macro を開発するときの Debugging の設定例
- [ ] rust-analyzer が Bug った時の挙動
  - https://github.com/rust-lang/rust-analyzer/issues/10894
- [ ] quote の役割は何か
- [ ] syn::Ident の役割は何か
- [ ] format_ident!() マクロは何か？
- [ ] cargo expand するとどのような内容が出力されるのか？
- [ ] span の出力は何か？
- [ ] span の call_site や def_site は何か？
- [ ] #fields を quote!内で利用した時にどのように展開されるのか？
- [ ] syn の parse_macro_input を使わなかった場合の出力は何か？
- [ ] Option の clone と take の違いは何か？

##

## サンプル

```rust
#[derive(Builder)]
struct Command {
  executable:String,
}
```

```bash
DeriveInput {
    attrs: [],
    vis: Visibility::Inherited,
    ident: Ident {
        ident: "Command",
        span: #0 bytes(56..63),
    },
    generics: Generics {
        lt_token: None,
        params: [],
        gt_token: None,
        where_clause: None,
    },
    data: Data::Struct {
        struct_token: Struct,
        fields: Fields::Named {
            brace_token: Brace,
            named: [
                Field {
                    attrs: [],
                    vis: Visibility::Inherited,
                    mutability: FieldMutability::None,
                    ident: Some(
                        Ident {
                            ident: "executable",
                            span: #0 bytes(70..80),
                        },
                    ),
                    colon_token: Some(
                        Colon,
                    ),
                    ty: Type::Path {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: [
                                PathSegment {
                                    ident: Ident {
                                        ident: "String",
                                        span: #0 bytes(82..88),
                                    },
                                    arguments: PathArguments::None,
                                },
                            ],
                        },
                    },
                },
                Comma,
            ],
        },
        semi_token: None,
    },
}
```
