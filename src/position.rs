use std::collections::BTreeMap;

use leptos::*;
use leptos_router::use_navigate;

use crate::book::Book;

pub fn init() {
    // selected page
    let (page, set_page) = create_signal(0usize);
    // map of what position we are on in every page
    let (pos, set_pos) = create_signal(BTreeMap::<usize, usize>::new());

    provide_context(page);
    provide_context(set_page);
    provide_context(pos);
    provide_context(set_pos);

    // hook: when book changes, load the saved position
    let book = expect_context::<ReadSignal<Book>>();
    create_effect(move |_| {
        let Ok(Some(storage)) = leptos::window().local_storage() else {
            return;
        };
        let Some(book) = book.get() else { return };
        let book = book.borrow();
        let id = book.identifier();
        let Ok(Some(saved_pos)) = storage.get_item(id) else {
            return;
        };
        let Some((page, para)) = saved_pos.split_once(':') else {
            return;
        };
        let page = page.parse::<usize>().unwrap();
        let para = para.parse::<usize>().unwrap();
        set_page.set(page);
        set_pos.update(|pos| _ = pos.insert(page, para));

        let Ok(Some(route)) = storage.get_item(&format!("{id}-route")) else {
            return;
        };

        if route == "content" {
            create_effect(move |_| {
                (use_navigate())(&page.to_string(), Default::default());
            });
        }
    });
}
