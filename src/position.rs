use std::collections::{BTreeMap, BTreeSet};

use leptos::*;
use wasm_bindgen::{closure::Closure, JsCast as _, JsValue};

use crate::{book::Book, config};

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
        if !config::get().borrow().save_position {
            return;
        }
        let Ok(Some(storage)) = leptos::window().local_storage() else {
            return;
        };
        let Some(book) = book.get() else { return };
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
    });
}

pub struct Tracker {
    obs: web_sys::IntersectionObserver,
    first_visible: Memo<Option<usize>>,
    pos: ReadSignal<BTreeMap<usize, usize>>,
}

impl Tracker {
    pub fn track(&self, node: HtmlElement<html::Div>, id: usize, page: usize) {
        self.obs.observe(&node);
        self.pos.with_untracked(move |pos| {
            if Some(id) == pos.get(&page).copied() && id != 0 {
                create_effect(move |_| {
                    node.scroll_into_view();
                });
            }
        });
    }

    pub fn first_visible(&self) -> Option<usize> {
        self.first_visible.get()
    }
}

pub fn init_tracking() -> Tracker {
    let (vs, set_vs) = create_signal(BTreeSet::<usize>::new());
    let first_visible = create_memo(move |_| vs.get().iter().min().copied());

    let book = expect_context::<ReadSignal<Book>>();
    let page = expect_context::<ReadSignal<usize>>();
    let pos = expect_context::<ReadSignal<BTreeMap<usize, usize>>>();
    let set_pos = expect_context::<WriteSignal<BTreeMap<usize, usize>>>();

    create_effect(move |prev| {
        let cur = page.get();
        if prev != Some(cur) {
            set_vs.set(Default::default());
        }
        cur
    });

    let config = config::get();
    create_effect(move |_| {
        let Some(para) = first_visible.get() else {
            return;
        };
        let Ok(Some(storage)) = leptos::window().local_storage() else {
            return;
        };
        let Some(book) = book.get() else { return };
        let page = page.get();
        let id = book.identifier();
        set_pos.update(move |pos| {
            if para > 1 {
                pos.insert(page, para);
            } else {
                pos.remove(&page);
            }
        });
        if config.borrow().save_position {
            let _ = storage.set_item(id, &format!("{page}:{para}"));
        }
    });

    let cb = move |entries, _| {
        for entry in entries {
            let entry = web_sys::IntersectionObserverEntry::from(entry);
            let id = entry.target().id().parse::<usize>().unwrap();
            match entry.is_intersecting() {
                true => set_vs.update(|vs| _ = vs.insert(id)),
                false => set_vs.update(|vs| _ = vs.remove(&id)),
            }
        }
    };

    let cb: Closure<dyn Fn(Vec<JsValue>, web_sys::IntersectionObserver)> = Closure::new(cb);
    let obs = web_sys::IntersectionObserver::new(cb.as_ref().unchecked_ref()).unwrap();
    let cleanup_obs = obs.clone();

    on_cleanup(move || {
        cleanup_obs.disconnect();
        drop(cb);
    });

    Tracker {
        obs,
        first_visible,
        pos,
    }
}
