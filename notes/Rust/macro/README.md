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

## 進め方

まずは本記事では `#[derive]` マクロを使って Builder パターンの実装を進めていき、最終的には以下のような処理を実現できるようにしていきます。

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

## 01-parse

まずは一番最初の課題である `01-parse` のテストコードでは、以下の `derive` マクロを利用したときにコンパイルエラーが発生しないようにしていきます。

```rust
#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}

fn main() {}
```

具体的には初期実装は以下のように `unimplemented!()` が利用されているため、関数の型シグネチャに合うように、空の実装を追加していきます。

```rust
use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;

    unimplemented!()
}
```

コンパイルを通すだけであれば空の `TokenStream` を返すために、以下のように空のトークンツリーを生成して返却すれば OK です。

```rust
use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;

    TokenStream::new()
}
```

- [proc_macro::TokenStream](https://doc.rust-lang.org/beta/proc_macro/struct.TokenStream.html)

`TokenStream` は Rust コードのトークンのストリームが含まれており、 `Command` 構造体の場合には以下のような内容が入力に含まれている。

```bash
TokenStream [
    Ident {
        ident: "struct",
        span: #0 bytes(39..45),
    },
    Ident {
        ident: "Command",
        span: #0 bytes(46..53),
    },
    Group {
        delimiter: Brace,
        stream: TokenStream [
            Ident {
                ident: "executable",
                span: #0 bytes(60..70),
            },
            Punct {
                ch: ':',
                spacing: Alone,
                span: #0 bytes(70..71),
            },
            Ident {
                ident: "String",
                span: #0 bytes(72..78),
            },
            # ... 残りの定義が続いていく
        ],
        span: #0 bytes(54..151),
    },
]
```

- [TokenStream の全文](https://gist.github.com/shimopino/e896b706c71949203d253ca7edd95b6e)

ただ、これはただのトークンのストリームでしかないため、Rust のソースコードの構文木にパースするための `syn` クレートも用意されている。

今回作成しているものは `derive` マクロであるため、 `syn::DeriveInput` という構造として解析することが可能である。

```rust
#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);

    TokenStream::new()
}
```

実際に構文木にパースした結果は以下のようになっており、Rust コードのトークンがツリー構造として変換されており、 `TokenStream` よりも取り扱いしやすい形式になっていることがわかります。

```bash
DeriveInput {
    attrs: [],
    vis: Visibility::Inherited,
    ident: Ident {
        ident: "Command",
        span: #0 bytes(46..53),
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
                            span: #0 bytes(60..70),
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
                                        span: #0 bytes(72..78),
                                    },
                                    arguments: PathArguments::None,
                                },
                            ],
                        },
                    },
                },
                # ...
            ],
        },
        semi_token: None,
    },
}
```

これでパターンマッチなどと合わせて細かい制御を行うことが可能となりました。

- [syn::DeriveInput](https://docs.rs/syn/latest/syn/struct.DeriveInput.html)
- [Command 構造体の DeriveInput](https://gist.github.com/shimopino/a5cf6c3810b3131b31ba99cc55074d5d)

他にもどのように構文木に解析されるのかが気になる場合は [AST Explorer](https://astexplorer.net/) を実際に触って様々なパターンを見てみるとよいと思います。

## 02-create-builder

次の課題は Builder の derive マクロを適用した構造体に対して、 `builder` メソッドを実装し、Builder パターンを実装するための準備を行います。

```rust
#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}

fn main() {
    let builder = Command::builder();

    let _ = builder;
}
```

ここでは手続きマクロの内部で以下のような構造体と、その構造体を生成するためのメソッドを作成します。

```rust
pub struct CommandBuilder {
    executable: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    current_dir: Option<String>,
}

impl Command {
    pub fn builder() -> CommandBuilder {
        CommandBuilder {
            executable: None,
            args: None,
            env: None,
            current_dir: None,
        }
    }
}
```

まずは汎用性などは無視してコンパイルエラーが発生しないようにするために、いくつかのプロパティはハードコードでそのまま生成する形式で進める。

手続きマクロの内部で Rust のコードを生成するときには `quote` クレートを利用すると簡単に生成することができる。

```rust
#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);

    let expanded = quote! {
        pub struct CommandBuilder {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl Command {
            pub fn builder() -> CommandBuilder {
                CommandBuilder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
    };

    // ここで TokenStream に型変換している
    expanded.into()
}
```

動作確認のために `cargo expand` を利用すれば、以下のように実際にマクロがどのように展開されているのかがわかり、今回ハードコードで設定した通りにソースコードが生成されていることがわかる。

```rust
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use demo::Builder;
struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}
pub struct CommandBuilder {
    executable: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    current_dir: Option<String>,
}
impl Command {
    pub fn builder() -> CommandBuilder {
        CommandBuilder {
            executable: None,
            args: None,
            env: None,
            current_dir: None,
        }
    }
}
fn main() {
    let builder = Command::builder();
    let _ = builder;
}
```

これでコンパイルエラーは発生せず、テストも PASS することができたが、ここで急に出てきた `quote` クレートの役割や、他の構造体でも適用できるようにするための汎用化の処理が不足している。

### `quote` クレート

最初の例で見たように、 入力となる `TokenStream` を実際に [ログに出力してみた結果](https://gist.github.com/shimopino/e896b706c71949203d253ca7edd95b6e) を確認すると、Rust のコードを表す値の配列となっていたことがわかる。

```bash
TokenStream [
    Ident {
        ident: "pub",
        span: #5 bytes(29..36),
    },
    Ident {
        ident: "struct",
        span: #5 bytes(29..36),
    },
    # ...
]
```

これは構文木を構成するトークンである `proc_macro::TokenTree` から構成されていることがわかる。

```rust
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}
```

- [proc_macro::TokenTree](https://doc.rust-lang.org/beta/proc_macro/enum.TokenTree.html)

例えば単純に以下のような構造体を追加で定義したいとする。

```rust
struct CommandBuilder {
    executable: String,
}
```

この構造体は `TokenTree` に分解するなら以下のように構成されることとなる。

![](assets/TokenTree.drawio.png)

この場合は `proc_macro::TokenTree` を利用すると以下のように定義することができる

```rust
use proc_macro::{Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {

    // IteratorTokenStream に変換するために
    [
        TokenTree::Ident(Ident::new("struct", Span::call_site())),
        TokenTree::Ident(Ident::new("CommandBuilder", Span::call_site())),
        TokenTree::Group(Group::new(
            proc_macro::Delimiter::Brace,
            [
                TokenTree::Ident(Ident::new("executable", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("String", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ]
            .into_iter()
            .collect::<TokenStream>(),
        )),
    ]
    .into_iter()
    .collect()
}
```

これで `cargo expand` を実行すれば、実際に以下のように設定したトークンに従って、Rust コードが生成されていることがわかる

```rust
struct CommandBuilder {
    executable: String,
}
fn main() {}
```

実は `quote!` はこれと似たようなことをより簡単に実行できるように用意されているクレートであり、実際に Rust のコードを記述すれば、それを `TokenStream` の形式に変換してくれる。

先ほどと同じことを `quote!` で実現したい場合には、以下のように単純に記述すれば、先ほど `TokenTree` を直接利用して記述していた内容と同じことを実現することができる。

```rust
#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // TokenTree を直接利用するよりも、はるかに簡易的に記述することができる
    let expanded = quote! {
        struct CommandBuilder {
            executable: String,
        }
    }

    // quote! が生成するのはライブラリ用に用意された proc_macto2::TokenStream なのでここで変換する
    expanded.into()
}
```

### Builder 構造体の名前の取得

今回の実装は `Command` 構造体に特化した実装になっていたが、他の構造体やプロパティ定義でも利用できるように汎用化させる必要がある。

具体的には Builder パターンの実装に関しては、以下のような構造体名とプロパティ定義のパターンが存在していることがわかる。

```rust
// 生成する構造体の名前のパターン [元の構造体の名前]Builder
pub struct CommandBuilder {
    // プロパティの型の定義のパターン [プロパティ名]: Option<元の型>,
    executable: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    current_dir: Option<String>,
}
```

つまり `syn` クレートを使用した `DeriveInput` にパースした後で、元の構造体の名前・構造体で定義されている各プロパティの名前と型さえ取得することができれば、汎用的な実装するにすることができる。

![](assets/DeriveInput.drawio.png)

- [全体像](https://gist.github.com/shimopino/a5cf6c3810b3131b31ba99cc55074d5d)

まずは構造体の名前を抽出し `[構造体の名前]Builder` という名前の Builder 用の構造体を作成する。

`quote!` 内部では識別子を単純に結合することはできないので、新しく `Ident` を作成して変数として利用する必要があり、以下のように 2 つのやり方が存在しています。

```rust
// quote::format_ident! を利用する方法
let ident = parsed.ident;
let builder_ident = format_ident!("{}Builder", ident);

// syn::Ident::new で直接生成する方法
let ident = parsed.ident;
let builder_name = format!("{}Builder", ident);
let builder_ident = syn::Ident::new(&builder_name, ident.span());

// どちらの場合でも quote! 内で利用できる
quote! {
    struct #builder_ident {
        // ...
    }

    impl #ident {
        pub fn builder() -> #builder_ident {
            #builder_ident {
                // ...
            }
        }
    }
}
```

これでどのような構造体に対して適用しても、対応する Builder 構造体を定義することができます。

- [constructing identifiers](https://docs.rs/quote/latest/quote/macro.quote.html#constructing-identifiers)

### Builder 構造体のプロパティの取得

これまで `quote!` を使って `TokenStream` を定義する際には個別に変数を指定したり、プロパティを指定したりしていたが、このマクロは内部で利用されたイテレータを展開してトークンツリーを組み立てることができる。

```rust
#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Iterator の検証のために動的にプロパティを作成するための元データを用意する
    let vars = vec!["a", "b", "c"];

    // 内部で quote! を使用して TokenStream の Iterator を用意する
    let delarations: Vec<proc_macro2::TokenStream> = vars
        .into_iter()
        .map(|var_name| {
            let ident = format_ident!("{}", var_name);
            // TokenStream を生成
            quote! {
                #ident: String,
            }
        })
        .collect();

    let expanded = quote! {
        pub struct Sample {
            // 以下のようにマクロ内で変数を展開するように指定することが可能である
            #(#delarations)*
        }
    };

    expanded.into()
}
```

ここで作成した内容を `cargo expand` で確認すれば、イテレータとして用意した変数を展開して全てのプロパティの定義を動的に作成できていることがわかる。

```rust
pub struct Sample {
    a: String,
    b: String,
    c: String,
}
```

- [quote! での Iterator 展開のテスト](https://github.com/dtolnay/quote/blob/d8cb63f7d7f45c503ac580bd8f3cb2d8bb28b160/tests/test.rs#L79-L87)

`CommandBuilder` の定義と `builder` メソッドの実装を作成する上では、同じように各プロパティや初期値を作成するためのイテレータを用意することを目指す。

```rust
#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = parsed.ident;
    let builder_ident = format_ident!("{}Builder", name);

    // 元が構造体であることと、タプルやUnit型を想定していないため、let else で対象データを抽出する
    let syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }), .. }) = parsed.data else {
        panic!("This macro can only be applied to struct using named field only, not tuple or unit.");
    };

    // 構造体を構成する各プロパティの定義には Field からアクセスすることが可能
    // Builder の定義と builder メソッドのそれぞれで必要なトークンの形に抽出する
    let builder_fields = named.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! {
            #ident: Option<#ty>
        }
    });

    let builder_init = named.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: None
        }
    });

    let expanded = quote! {
        pub struct #builder_ident {
            #(#builder_fields,)*
        }

        impl #name {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_init,)*
                }
            }
        }
    };

    expanded.into()
}
```

これを展開すれば、以下のようにイテレーターが展開されてそれぞれの定義が作成されていることがわかる。

```rust
pub struct CommandBuilder {
    executable: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    current_dir: Option<String>,
}
impl Command {
    pub fn builder() -> CommandBuilder {
        CommandBuilder {
            executable: None,
            args: None,
            env: None,
            current_dir: None,
        }
    }
}
```

これで他の構造体に対しても適用することが可能な汎用的なマクロにすることができました。

## 03-call-setters

次の課題では以下のように `Command` で定義されている各プロパティに対して値を設定するための `setter` を準備します

```rust
#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}

fn main() {
    let mut builder = Command::builder();
    // プロパティと同じ名称で同じ型を引数に受け取るメソッドを用意する
    builder.executable("cargo".to_owned());
    builder.args(vec!["build".to_owned(), "--release".to_owned()]);
    builder.env(vec![]);
    builder.current_dir("..".to_owned());
}
```

これは構造としては以下のパターンに従うメソッドを作成することと同義です。

```rust
fn [プロパティ名](&mut self, [プロパティ名]: [プロパティの型]) -> &mut Self {
    self.[プロパティ名] = Some([プロパティ名])
    self
}
```

前回の課題のときの汎用化の際に適用した実装の大部分を流用するだけで実装することができます。

```rust
let builder_setters = named.iter().map(|f| {
    let ident = &f.ident;
    let ty = &f.ty;
    quote! {
        fn #ident(&mut self, #ident: #ty) -> &mut Self {
            self.#ident = Some(#ident);
            self
        }
    }
});

let expanded = quote! {
    pub struct #builder_ident {
        #(#builder_fields,)*
    }

    impl #builder_ident {
        #(#builder_setters)*
    }

    // ...
};
```

これで以下のように展開すれば指定した通りのメソッドが作成されていることがわかる。

```rust
impl CommandBuilder {
    fn executable(&mut self, executable: String) -> &mut Self {
        self.executable = Some(executable);
        self
    }
    fn args(&mut self, args: Vec<String>) -> &mut Self {
        self.args = Some(args);
        self
    }
    fn env(&mut self, env: Vec<String>) -> &mut Self {
        self.env = Some(env);
        self
    }
    fn current_dir(&mut self, current_dir: String) -> &mut Self {
        self.current_dir = Some(current_dir);
        self
    }
}
```

## 04-call-build

次の課題では以下のように `build` メソッドを作成し、全てのプロパティに値が設定されている場合には `Ok(Command)` を返却し、設定されていないものがあれば `Err` を返却するようにします。

```rust
fn main() {
    let mut builder = Command::builder();
    builder.executable("cargo".to_owned());
    builder.args(vec!["build".to_owned(), "--release".to_owned()]);
    builder.env(vec![]);
    builder.current_dir("..".to_owned());

    // build() メソッドから Result を返却する
    // 設定されていないプロパティがある場合には Err を返却する
    let command = builder.build().unwrap();
    assert_eq!(command.executable, "cargo");
}
```

`Command` のプロパティの場合には、以下のように実装すれば良さそうです。

```rust
impl CommandBuilder {
    fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
        Ok(Command {
            executable: self.executable.take().ok_or("executable is not set")?,
            // ...
        })
    }
}
```

ただしこれはあくまで一例であり、以下のような実装パターンがあります。

1. `&mut self` で参照として受け取り、データを `take` する
   - データを `Command` に移動させながら、ビルダー自体は再利用することができる
   - ビルダー側の値は `None` にリセットされるため、注意して利用する必要がある
2. `&self` で参照として受け取り、データを `clone` する
   - ビルダーには変更されないのでそのまま再利用できる
   - データのコピーが必要であり、パフォーマンスに影響を与える可能性がある

今回は値を移動させるためにするのでパターン 1 で実装します。

```rust
let build_fields = named.iter().map(|f| {
    let ident = &f.ident;
    quote! {
        #ident: self.#ident.take().ok_or(format!("{} is not set", stringify!(#ident)))?
    }
});

let expanded = quote! {
    pub struct #builder_ident {
        // ...
    }

    impl #builder_ident {
        // ...
        fn build(&mut self) -> Result<#ident, Box<dyn std::error::Error>> {
            Ok(#ident {
                #(#build_fields,)*
            })
        }
    }
};
```

これでコンパイルエラーが発生することなくメソッドを追加できました。

## 05-method-chaining

次の課題はメソッドチェーン形式での Builder の利用ではあるが、すでにこれまでの課題が完了していれば問題なくコンパイルできる。

```rust
fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        .current_dir("..".to_owned())
        .build()
        .unwrap();

    assert_eq!(command.executable, "cargo");
}
```

## 疑問点

- [ ] trybuild とは何か？
- [ ] Cargo.toml における test とは何か？
- [ ] 外部クレートを利用せずに実装するには？
- [ ] proc_macro を開発するときの Debugging の設定例
- [ ] rust-analyzer が Bug った時の挙動
  - https://github.com/rust-lang/rust-analyzer/issues/10894
- [ ] span の出力は何か？
- [ ] span の call_site や def_site は何か？
- [ ] Option の clone と take の違いは何か？
