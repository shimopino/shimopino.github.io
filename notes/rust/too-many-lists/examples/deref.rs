use std::ops::Deref;

fn main() {
    let boxed_five = Box::new(5);
    assert_eq!(boxed_five.deref(), &5);
    assert_eq!(*boxed_five, 5);

    let some_boxed_five = Some(boxed_five);
    assert_eq!(some_boxed_five.as_deref(), Some(&5));
}
