use std::{cell::RefCell, rc::Rc};

use leptos::*;
use leptos_router::*;
use lepu::{Chapter, Content, Epub, Style, Text, TextKind};
use wasm_bindgen::{closure::Closure, JsCast as _, JsValue};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

type BookResource = Resource<Option<web_sys::File>, Option<Rc<RefCell<Epub>>>>;

#[derive(Default, Clone, Copy)]
struct Position {
    page: usize,
    para: Option<usize>,
}

#[component]
fn App() -> impl IntoView {
    let (source, set_source) = create_signal(None);
    let (pos, set_pos) = create_signal(Position::default());
    let res = create_local_resource(
        move || source.get(),
        move |file: Option<web_sys::File>| async move {
            use web_sys::js_sys::Uint8Array;
            let file = file?;
            let promise = file.array_buffer();
            let future = wasm_bindgen_futures::JsFuture::from(promise);
            let res = future.await.ok()?;
            let buf = Uint8Array::new(&res).to_vec();
            let epub = Epub::new(buf).ok()?;

            if let Ok(Some(storage)) = leptos::window().local_storage() {
                let id = epub.identifier();
                if let Ok(Some(saved_pos)) = storage.get_item(id) {
                    if let Some((page, para)) = saved_pos.split_once(':') {
                        let page = page.parse::<usize>().unwrap();
                        let para = para.parse::<usize>().unwrap();
                        set_pos.set(Position {
                            page,
                            para: Some(para),
                        });
                        (use_navigate())(&page.to_string(), Default::default());
                    }
                }
            }
            Some(Rc::new(RefCell::new(epub)))
        },
    );

    provide_context(pos);
    provide_context(set_pos);
    provide_context(res);

    let main_view = move || {
        let book_exists = move || matches!(res.get(), Some(Some(_)));
        let upload_view = move || {
            view! { <Upload file=set_source /> }
        };
        view! {
            <Show when=book_exists fallback=upload_view>
                <Outlet/>
            </Show>
        }
    };

    view! {
        <Router base="/wepu">
        <main>
            <div class="flex flex-col max-w-screen-sm xl:max-w-screen-md min-h-screen mx-auto space-y-10 py-10 text-2xl">
                <Routes base="/wepu".to_string()>
                    <Route path="/" view=main_view>
                        <Route path="" view=Navigation />
                        <Route path=":idx" view=Content />
                    </Route>
                </Routes>
            </div>
        </main>
        </Router>
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
            <input on:input=on_submit class="hidden" tabindex=-1 type="file" id="upload" name="book" accept="application/epub+zip" node_ref=input_element />
        </div>
    }
}

#[component]
fn Navigation() -> impl IntoView {
    fn make_list<'a>(entries: impl Iterator<Item = &'a Chapter>) -> leptos::View {
        let pos = expect_context::<ReadSignal<Position>>();
        let inner = entries
            .map(|e: &Chapter| {
                let sublist = e.has_children().then(|| make_list(e.children()));
                let idx = e.index_in_spine().to_string();
                let name = e.name().to_owned();
                let class = if e.index_in_spine() == pos.get().page {
                    "text-sky-400"
                } else {
                    "text-sky-300"
                };
                view! {
                    <li><A href=idx class=class>{name}</A>
                        {sublist}
                    </li>
                }
            })
            .collect_view();

        html::ul()
            .classes("ml-5 space-y-2 pt-2 pb-4")
            .child(inner)
            .into_view()
    }

    view! {
        { move || {
            let book = expect_context::<BookResource>().get().unwrap().unwrap();
            let b = book.borrow();
            view! {
                <div class="pt-8 pb-6 text-left font-sans font-bold text-4xl tracking-tight leading-none">
                    <h1>{b.title().to_owned()}</h1>
                </div>
                <div class="text-justify font-serif">
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
        TextKind::Header => html::div()
            .attr(
                "class",
                "mt-8 mb-6 text-left font-sans font-bold text-4xl tracking-tight leading-none",
            )
            .child(html::h1().child(children))
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
    let params = use_params::<ChapterParams>();
    let pos = expect_context::<ReadSignal<Position>>();
    let set_pos = expect_context::<WriteSignal<Position>>();
    let book = expect_context::<BookResource>();
    let page = move || match params.with(|p| p.as_ref().map(|p| p.idx).ok().flatten()) {
        Some(page) => page,
        None => {
            // HACK: trailing slash makes this route match
            (use_navigate())(
                "/",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
            0
        }
    };

    // FIXME: not meant to write to signals within effects
    create_effect(move |_| set_pos.update(|pos| pos.page = page()));

    let (vs, set_vs) = create_signal(std::collections::BTreeSet::<usize>::new());
    let first_visible_block = create_memo(move |_| vs.get().iter().min().copied());

    create_effect(move |_| {
        let Some(para) = first_visible_block.get() else {
            return;
        };
        let Ok(Some(storage)) = leptos::window().local_storage() else {
            return;
        };
        let Some(Some(book)) = book.get() else { return };
        let book = book.borrow();
        let id = book.identifier();
        let page = page();
        if para != 0 {
            set_pos.update(move |pos| pos.para = Some(para));
        }
        if let Err(e) = storage.set_item(id, &format!("{page}:{para}")) {
            logging::log!("failed to set local storage: {e:?}");
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

    {
        let clean_obs = obs.clone();
        on_cleanup(move || {
            clean_obs.disconnect();
            drop(cb)
        });
    }

    let text = move || {
        let mut out: Vec<View> = Vec::new();

        let mut id = 0;
        let book = book.get().unwrap().unwrap();
        book.borrow_mut()
            .traverse_chapter(page(), |content, _| {
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
                        if Some(id) == pos.get_untracked().para && id != 0 {
                            create_effect(move |_| {
                                node.scroll_into_view();
                            });
                        }
                    });
                out.push(view.into_view());
                id += 1;
            })
            .unwrap();

        Some(out.into_iter().collect_view())
    };

    let navigate = use_navigate();
    let handle = window_event_listener(ev::keyup, move |ev: ev::KeyboardEvent| {
        if ev.alt_key() || ev.shift_key() || ev.meta_key() || ev.ctrl_key() {
            return;
        }
        let move_page = |id| {
            set_pos.set(Position {
                page: id,
                para: None,
            });
            set_vs.set(Default::default());
            navigate(&id.to_string(), Default::default());
        };
        match &*ev.key() {
            "ArrowLeft" => {
                let id = page();
                if id == 0 {
                    return;
                };
                move_page(id - 1);
            }
            "ArrowRight" => {
                let id = page();
                let max_id = book.get().unwrap().unwrap().borrow().document_count();
                if id + 1 >= max_id {
                    return;
                };
                move_page(id + 1);
            }
            "Escape" => navigate("/", Default::default()),
            _ => {}
        }
    });
    on_cleanup(move || handle.remove());

    view! {
        {move || {
            view! {
                <div class="text-justify font-serif font-light space-y-5
                            tracking-tight leading-tight">
                    {text()}
                </div>
            }
        }}
    }
}
