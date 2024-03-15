use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use leptos::*;

pub type Marks = Rc<RefCell<BTreeMap<char, (usize, usize)>>>;

pub fn init() {
    let marks = Rc::new(RefCell::new(BTreeMap::new()));
    provide_context::<Marks>(marks);
}
