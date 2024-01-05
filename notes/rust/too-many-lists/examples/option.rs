fn main() {
    let some_string1 = Some(String::from("hello world"));
    let some_string_len1 = some_string1.as_ref().map(|s| s.len());

    println!("original string: {:?}", some_string1);
    println!("length string: {:?}", some_string_len1);

    // --- --- ---

    let some_string2 = Some(String::from("hello world"));
    let some_string_len2 = some_string2.map(|s| s.len());

    // borrow of moved value: `some_string` value borrowed here after move
    // println!("original string: {:?}", some_string2);
    println!("length string: {:?}", some_string_len2);
}
