use std::{cell::RefCell, rc::Rc};

use base64::prelude::*;
use leptos::*;
use leptos_router::use_navigate;
use lepu::Epub;

use crate::config;

pub type Book = Option<Rc<RefCell<Epub>>>;

pub fn init() {
    let (book, set_book) = create_signal::<Option<Rc<RefCell<Epub>>>>(None);
    let (source, set_source) = create_signal(None);

    let config = config::get();
    let config_ = config.clone();
    let res = create_local_resource(
        move || source.get(),
        move |file: Option<web_sys::File>| async move {
            use web_sys::js_sys::Uint8Array;
            let file = file?;
            let promise = file.array_buffer();
            let future = wasm_bindgen_futures::JsFuture::from(promise);
            let res = future.await.ok()?;
            Some(Uint8Array::new(&res).to_vec())
        },
    );

    // set book in response to change in resource
    create_effect(move |_| {
        let Some(Some(buf)) = res.get() else { return };
        let encoded = BASE64_STANDARD.encode(&buf);
        let Ok(epub) = Epub::new(buf) else { return };
        if config_.borrow().cache_book {
            if let Ok(Some(storage)) = leptos::window().local_storage() {
                if encoded.len() < 3_000_000 {
                    let _ = storage.set_item("b", &encoded);
                } else {
                    let _ = storage.remove_item("b");
                }
            }
        }
        // reset the resource to save some memory
        set_source.set(None);
        set_book.set(Some(Rc::new(RefCell::new(epub))));
    });

    // load book from local storage
    if config.borrow().cache_book {
        if let Ok(Some(storage)) = leptos::window().local_storage() {
            if let Ok(Some(saved_book)) = storage.get_item("b") {
                if let Ok(data) = BASE64_STANDARD.decode(saved_book) {
                    let epub = Epub::new(data).ok().unwrap();
                    set_book.set(Some(Rc::new(RefCell::new(epub))));
                }
            }
        }
    }

    provide_context(set_source);
    provide_context(book);
    provide_context(set_book);
}

pub fn unload() {
    let set_book = expect_context::<WriteSignal<Book>>();
    set_book.set(None);
    (use_navigate())("", Default::default());
    if let Ok(Some(storage)) = leptos::window().local_storage() {
        let _ = storage.remove_item("b");
    }
}
