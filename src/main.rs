use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use base64::prelude::*;
use leptos::*;
use leptos_router::*;
use lepu::{Chapter, Content, Epub, Style, Text, TextKind};
use wasm_bindgen::{closure::Closure, JsCast as _, JsValue};

mod input;

// TODO:
// - mobile styling & usability (touch regions on left and right sides to move pages)

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

type Book = Option<Rc<RefCell<Epub>>>;
type Marks = Rc<RefCell<BTreeMap<char, (usize, usize)>>>;

// local storage usage (non-normative)
// "b" => base64 encoded epub (most recently loaded book)
// "{book identifier}" => "{page}:{para}" (current position)
// "{book identifier}-route" => "nav" | "content" (current route)
// "c" => "{true|false}:{true|false}" (config fields in order)

#[derive(Debug)]
struct Config {
    save_position: bool,
    cache_book: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_position: true,
            cache_book: false,
        }
    }
}

fn load_config() -> Option<Config> {
    let Ok(Some(storage)) = leptos::window().local_storage() else {
        return None;
    };
    let Ok(Some(config_string)) = storage.get_item("c") else {
        return None;
    };

    logging::log!("loaded: {config_string}");
    let (save_position, cache_book) = config_string.split_once(':')?;
    Some(Config {
        save_position: save_position.parse::<bool>().ok()?,
        cache_book: cache_book.parse::<bool>().ok()?,
    })
}

fn save_config(config: &Config) {
    let Ok(Some(storage)) = leptos::window().local_storage() else {
        return;
    };

    let config_string = format!("{}:{}", config.save_position, config.cache_book);
    logging::log!("{config_string}");
    let _ = storage.set_item("c", &config_string);
}

#[component]
fn App() -> impl IntoView {
    let (source, set_source) = create_signal(None);
    // selected page
    let (page, set_page) = create_signal(0usize);
    // map of what position we are on in every page
    let (pos, set_pos) = create_signal(BTreeMap::<usize, usize>::new());
    // map of bookmarks
    let marks = Rc::new(RefCell::new(BTreeMap::new()));
    provide_context::<Marks>(marks);

    let config = Rc::new(RefCell::new(load_config().unwrap_or_default()));
    provide_context(config.clone());

    let load_saved_pos = move |epub: &Epub| {
        if let Ok(Some(storage)) = leptos::window().local_storage() {
            let id = epub.identifier();
            if let Ok(Some(saved_pos)) = storage.get_item(id) {
                if let Some((page, para)) = saved_pos.split_once(':') {
                    let page = page.parse::<usize>().unwrap();
                    let para = para.parse::<usize>().unwrap();
                    set_page.set(page);
                    set_pos.update(|pos| _ = pos.insert(page, para));
                    if let Ok(Some(route)) = storage.get_item(&format!("{id}-route")) {
                        if route == "content" {
                            create_effect(move |_| {
                                (use_navigate())(&page.to_string(), Default::default());
                            });
                        }
                    }
                }
            }
        }
    };

    let (book, set_book) = create_signal::<Option<Rc<RefCell<Epub>>>>(None);

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

    let config_ = config.clone();
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
        if config_.borrow().save_position {
            load_saved_pos(&epub);
        }
        // FIXME: not meant to write to signals within effects
        // reset the resource to save some memory
        set_source.set(None);
        set_book.set(Some(Rc::new(RefCell::new(epub))));
    });

    if config.borrow().cache_book {
        if let Ok(Some(storage)) = leptos::window().local_storage() {
            if let Ok(Some(saved_book)) = storage.get_item("b") {
                if let Ok(data) = BASE64_STANDARD.decode(saved_book) {
                    let epub = Epub::new(data).ok().unwrap();
                    load_saved_pos(&epub);
                    set_book.set(Some(Rc::new(RefCell::new(epub))));
                }
            }
        }
    }

    provide_context(page);
    provide_context(set_page);
    provide_context(pos);
    provide_context(set_pos);
    provide_context(book);
    provide_context(set_book);

    let main_view = move || {
        let book_exists = move || matches!(book.get(), Some(_));
        let upload_view = move || {
            view! { <Upload file=set_source /> }
        };
        view! {
            <Show when=book_exists fallback=upload_view>
                <nav>
                    <div class="flex justify-between px-1 text-sm md:text-base">
                        <button on:click=move |_| unload_book()>"✕"</button>
                        <ul class="flex space-x-6 md:space-x-10">
                            <li><A href={move || format!("{}", page.get())}>read</A></li>
                            <li><A href="">table of contents</A></li>
                            <li><A href="settings">settings</A></li>
                        </ul>
                    </div>
                </nav>
                <Outlet/>
            </Show>
        }
    };

    view! {
        <Router base="/wepu">
            <main>
                <div class="flex flex-col max-w-screen-sm md:max-w-screen-md
                            min-h-screen mx-auto px-2 py-2 md:py-10 text-lg md:text-2xl">
                    <Routes base="/wepu".to_string()>
                        <Route path="/" view=main_view>
                            <Route path="" view=Navigation />
                            <Route path="settings" view=Settings />
                            <Route path=":idx" view=Content />
                        </Route>
                    </Routes>
                </div>
            </main>
        </Router>
    }
}

#[component]
fn Settings() -> impl IntoView {
    let clear_storage = || {
        if let Ok(Some(storage)) = leptos::window().local_storage() {
            let _ = storage.clear();
        }
    };

    let config = expect_context::<Rc<RefCell<Config>>>();
    let config_ = config.clone();
    let save_position = move |ev| {
        config_.borrow_mut().save_position = event_target_checked(&ev);
        save_config(&*config_.borrow());
    };
    let config_ = config.clone();
    let cache_book = move |ev| {
        let checked = event_target_checked(&ev);
        config_.borrow_mut().cache_book = checked;
        save_config(&*config_.borrow());
        if !checked {
            if let Ok(Some(storage)) = leptos::window().local_storage() {
                let _ = storage.remove_item("b");
            }
        }
    };

    view! {
        <h1 class="mt-8 mb-10 text-left font-sans font-bold text-2xl md:text-4xl tracking-tight leading-none">
            Settings
        </h1>

        <div class="flex flex-col justify-center text-base space-y-3">
        <label class="inline-flex items-center">
            <input type="checkbox" class="rounded-xs text-sky-500" id="save-position" checked={config.borrow().save_position} on:input=save_position/>
            <span class="ml-2">Save book position between sessions</span>
        </label>
        <label class="inline-flex items-center">
            <input type="checkbox" class="rounded-xs text-sky-500" id="cache-book" checked={config.borrow().cache_book} on:input=cache_book/>
            <span class="ml-2">"Save most recent book between sessions (if book smaller than 3 megabytes)"</span>
        </label>
            <div><button class="bg-zinc-200 text-zinc-800 mt-2 active:bg-sky-500 active:text-zinc-200 rounded-lg px-3 py-1" on:click=move |_| clear_storage()>Clear data</button></div>
        </div>
    }
}

fn unload_book() {
    let set_book = expect_context::<WriteSignal<Book>>();
    set_book.set(None);
    if let Ok(Some(storage)) = leptos::window().local_storage() {
        let _ = storage.remove_item("b");
    }
}

#[component]
fn Upload(file: WriteSignal<Option<web_sys::File>>) -> impl IntoView {
    let input_element: NodeRef<html::Input> = create_node_ref();
    let on_submit = move |ev: ev::Event| {
        ev.prevent_default();
        let e = input_element.get().unwrap();
        let input_file = e.files().unwrap().get(0).unwrap();
        file.set(Some(input_file));
    };
    let on_click = move |_| {
        let e = input_element.get().unwrap();
        e.click();
    };
    view! {
        <div class="flex flex-1 justify-center items-center font-sans text-base">
            <button on:click:undelegated=on_click
                class="py-1 px-6 font-sans mb-2 rounded-lg block text-center
                       bg-1 border-dashed border-zinc-600 border-2
                       aspect-square w-4/12 active:bg-zinc-800">
            </button>
            <input
                on:input=on_submit
                class="hidden"
                tabindex=-1
                type="file"
                id="upload"
                name="book"
                accept="application/epub+zip"
                node_ref=input_element />
        </div>
    }
}

#[component]
fn Navigation() -> impl IntoView {
    let config = expect_context::<Rc<RefCell<Config>>>();
    if config.borrow().save_position {
        create_effect(move |_| {
            if let Ok(Some(storage)) = leptos::window().local_storage() {
                let Some(book) = expect_context::<ReadSignal<Book>>().get() else {
                    return;
                };
                let book = book.borrow();
                let id = book.identifier();
                let _ = storage.set_item(&format!("{id}-route"), "nav");
            }
        });
    }

    fn make_list<'a>(entries: impl Iterator<Item = &'a Chapter>) -> leptos::View {
        let page = expect_context::<ReadSignal<usize>>();
        let inner = entries
            .map(|e: &Chapter| {
                let sublist = e.has_children().then(|| make_list(e.children()));
                let idx = e.index_in_spine().to_string();
                let name = e.name().to_owned();
                let class = if e.index_in_spine() == page.get() {
                    "text-sky-500"
                } else {
                    "text-zinc-200"
                };
                view! {
                    <li><div class="pb-2"><A href=idx class=class>{name}</A></div>
                        {sublist}
                    </li>
                }
            })
            .collect_view();

        html::ul().classes("ml-5").child(inner).into_view()
    }

    view! {
        { move || {
            let book = expect_context::<ReadSignal<Book>>().get().unwrap();
            let b = book.borrow();
            view! {
                <div class="mt-8 mb-10 text-left font-bold text-2xl md:text-4xl tracking-tight leading-none">
                    <h1>{b.title().to_owned()}</h1>
                </div>
                <div class="text-justify font-serif tracking-tight leading-tight">
                    {make_list(b.chapters())}
                </div>
            }
        }}
    }
}

fn convert(text: &Text<'_>) -> leptos::View {
    let mut children = Vec::new();
    for (slice, style) in text.style_chunks() {
        let mut views = Vec::new();
        for (i, chunk) in slice.split('\n').enumerate() {
            if i > 0 {
                views.push(html::br().into_view());
            }
            if !chunk.is_empty() {
                views.push(chunk.to_owned().into_view());
            }
        }
        let mut view = views.collect_view();
        if style.contains(Style::ITALIC) {
            view = html::i().child(view).into_view();
        }
        if style.contains(Style::BOLD) {
            view = html::b().child(view).into_view();
        }
        children.push(view);
    }

    match text.kind() {
        TextKind::Header => html::h1()
            .attr(
                "class",
                "mb-6 md:mb-10 text-left font-sans font-bold text-2xl md:text-4xl tracking-tight leading-none",
            )
            .child(children)
            .into_view(),
        TextKind::Paragraph => html::p().child(children).into_view(),
        TextKind::Quote => html::blockquote()
            .attr("class", "mx-12")
            .child(children)
            .into_view(),
    }
}

#[derive(Params, PartialEq)]
struct ChapterParams {
    idx: Option<usize>,
}

#[component]
fn Content() -> impl IntoView {
    let config = expect_context::<Rc<RefCell<Config>>>();
    if config.borrow().save_position {
        create_effect(move |_| {
            if let Ok(Some(storage)) = leptos::window().local_storage() {
                let Some(book) = expect_context::<ReadSignal<Book>>().get() else {
                    return;
                };
                let book = book.borrow();
                let id = book.identifier();
                let _ = storage.set_item(&format!("{id}-route"), "content");
            }
        });
    }

    let params = use_params::<ChapterParams>();
    let pos = expect_context::<ReadSignal<BTreeMap<usize, usize>>>();
    let set_pos = expect_context::<WriteSignal<BTreeMap<usize, usize>>>();
    let cur_page = expect_context::<ReadSignal<usize>>();
    let set_page = expect_context::<WriteSignal<usize>>();
    let book = expect_context::<ReadSignal<Book>>();
    let param_page = move || match params.with(|p| p.as_ref().map(|p| p.idx).ok().flatten()) {
        Some(page) => page,
        None => {
            if book.get().is_none() {
                // HACK: trailing slash makes this route match
                (use_navigate())(
                    "/",
                    NavigateOptions {
                        replace: true,
                        ..Default::default()
                    },
                );
            }
            0
        }
    };

    // FIXME: not meant to write to signals within effects
    create_effect(move |_| set_page.set(param_page()));

    let (vs, set_vs) = create_signal(BTreeSet::<usize>::new());
    let first_visible_block = create_memo(move |_| vs.get().iter().min().copied());

    let config = expect_context::<Rc<RefCell<Config>>>();
    create_effect(move |_| {
        let Some(para) = first_visible_block.get() else {
            return;
        };
        let Ok(Some(storage)) = leptos::window().local_storage() else {
            return;
        };
        let Some(book) = book.get() else { return };
        let book = book.borrow();
        let id = book.identifier();
        let page = param_page();
        if para != 0 {
            // FIXME: not meant to write to signals within effects
            set_pos.update(move |pos| _ = pos.insert(page, para));
        }
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

    let text = move || {
        let mut out: Vec<View> = Vec::new();

        let mut id = 0;
        let book = book.get().unwrap();
        let page = param_page();
        let cur_page = cur_page.get();
        book.borrow_mut()
            .traverse_chapter(page, |content, _| {
                let view = match content {
                    Content::Textual(tc) => convert(&tc).into_view(),
                    Content::Image => view! {}.into_view(),
                };
                let obs = obs.clone();
                let view = html::div()
                    .id(id.to_string())
                    .child(view)
                    .on_mount(move |node| {
                        obs.observe(&node);
                        pos.with_untracked(move |pos| {
                            if Some(id) == pos.get(&page).copied() && id != 0 && page == cur_page {
                                create_effect(move |_| {
                                    node.scroll_into_view();
                                });
                            }
                        });
                    });
                out.push(view.into_view());
                id += 1;
            })
            .unwrap();

        Some(out.into_iter().collect_view())
    };

    let leave = move || {
        set_vs.set(Default::default());
        (use_navigate())("/", Default::default());
    };

    let navigate = use_navigate();
    let handler = RefCell::new(input::Handler::new());
    let marks = expect_context::<Marks>();
    let handle = window_event_listener(ev::keyup, move |ev: ev::KeyboardEvent| {
        if ev.alt_key() || ev.shift_key() || ev.meta_key() || ev.ctrl_key() {
            return;
        }
        let move_page = |id| {
            set_page.set(id);
            set_vs.set(Default::default());
            navigate(&id.to_string(), Default::default());
        };

        let Some(action) = handler.borrow_mut().handle(&*ev.key()) else {
            return;
        };

        use input::Action;
        match action {
            Action::NextPage => {
                let id = param_page();
                let max_id = book.get().unwrap().borrow().document_count();
                if id + 1 < max_id {
                    move_page(id + 1);
                }
            }
            Action::PreviousPage => {
                let id = param_page();
                if id > 0 {
                    move_page(id - 1);
                }
            }
            Action::SetMark(c) => {
                let Some(para) = first_visible_block.get() else {
                    return;
                };
                let page = param_page();
                marks.borrow_mut().insert(c, (page, para));
            }
            Action::FollowMark(c) => {
                if let Some((page, para)) = marks.borrow().get(&c).copied() {
                    set_pos.update(move |pos| _ = pos.insert(page, para));
                    move_page(page);
                }
            }
            Action::Leave => leave(),
        }
    });

    on_cleanup(move || {
        cleanup_obs.disconnect();
        handle.remove();
        drop(cb);
    });

    view! {
        {move || {
            view! {
                <div class="text-justify font-serif font-light space-y-3 md:space-y-5
                            tracking-tight leading-tight mt-8">
                    {text()}
                </div>
            }
        }}
    }
}
