fn main() {
    let mut v = vec![1, 2];
    let old_v = std::mem::take(&mut v);

    assert_eq!(v, vec![]);
    assert_eq!(old_v, vec![1, 2]);
}
