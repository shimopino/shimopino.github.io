struct Configuration {
    _name: String,
    _age: u8,
}

fn main() {
    let conf = Configuration {
        _name: "shimopino".to_string(),
        _age: 30,
    };
    let _some_conf = Some(conf);

    // Configuration 型は Clone トレイトを実装していないため、以下の呼び出しはコンパイルエラーとなる
    // _some_conf.clone()
}
