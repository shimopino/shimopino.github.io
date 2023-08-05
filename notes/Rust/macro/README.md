# Rustのマクロをマスターする

## 疑問点
- [ ] trybuildとは何か？
- [ ] Cargo.tomlにおけるtestとは何か？
- [ ] 外部クレートを利用せずに実装するには？
- [ ] parse_macro_input
- [ ] DeriveInput
- [ ] proc_macro_deriveとその他のマクロとの違いは何か？
- [ ] 単純な `Command` をBuilderマクロを適用した時の `DeriveInput` の具体的なDebug内容（ `{:#?}` ）
- [ ] proc_macro を開発するときのDebuggingの設定例
- [ ] rust-analyzerがBugった時の挙動
  - https://github.com/rust-lang/rust-analyzer/issues/10894
- [ ] quoteの役割は何か
- [ ] syn::Identの役割は何か
- [ ] format_ident!() マクロは何か？
- [ ] cargo expandするとどのような内容が出力されるのか？
- [ ] spanの出力は何か？
- [ ] spanのcall_siteやdef_siteは何か？
- [ ] 

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
