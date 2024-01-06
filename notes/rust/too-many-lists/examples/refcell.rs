use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

fn main() {
    let shared_map = Rc::new(RefCell::new(HashMap::new()));
    shared_map.borrow_mut().insert("africa", 92388);
    assert_eq!(shared_map.borrow().get("africa"), Some(&92388));

    let _ref_hash = shared_map.borrow();
    let _ref_hash_map = Ref::map(_ref_hash, |hash| hash);
}
