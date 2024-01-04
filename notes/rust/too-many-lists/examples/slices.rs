fn first_word_v1(s: &String) -> usize {
    let bytes = s.as_bytes();

    for (index, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return index;
        }
    }

    s.len()
}

fn first_word_v2(s: &String) -> &str {
    let bytes = s.as_bytes();

    for (index, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..index];
        }
    }

    &s[..]
}

fn first_word_v3(s: &str) -> &str {
    let bytes = s.as_bytes();

    for (index, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..index];
        }
    }

    s
}

fn main() {
    {
        let mut s = String::from("hello world");
        // もはやこの word は s と何の関係もなくなったが、そのことはコンパイラは何もわからない
        let word = first_word_v1(&s);
        s.clear();

        println!("index {}", word);
    }

    {
        let mut s = String::from("hello world");
        let word = first_word_v2(&s);
        // println! で不変参照を取得しているため、可変参照を必要とするメソッドを使えない （borrow checker　により）
        // cannot borrow `s` as mutable because it is also borrowed as immutable mutable borrow occurs here
        // s.clear();

        println!("string slice {}", word);
    }

    {
        let my_string = String::from("hello world");

        // String 型に対するスライス指定でも機能する
        let word = first_word_v3(&my_string[0..6]);
        let word = first_word_v3(&my_string[..]);

        // String 型自体への参照は全インデックスに対するスライスと同じ
        let word = first_word_v3(&my_string);

        let my_string_literal = "hello world";

        // 文字列リテラルに対するスライスであっても機能する
        let word = first_word_v3(&my_string_literal[0..6]);
        let word = first_word_v3(&my_string_literal[..]);

        // 文字列スライスは専用のメモリ空間に生成されたデータに対するスライスである
        let word = first_word_v3(my_string_literal);

        println!("word: {}", word);
    }

    {
        let word = String::from("あ");
        println!("あ length: {}", word.len());

        // panic !!!
        println!("あ slices: {}", &word[0..2]);
    }
}
