fn main() {
    let some_string = Some(String::from("hello world"));
    let some_string_len = some_string.map(|s| s.len());

    // borrow of moved value: `some_string` value borrowed here after move
    // println!("original string: {:?}", some_string);
    println!("length string: {:?}", some_string_len);
}
