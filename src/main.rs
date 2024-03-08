use leptos::*;
use leptos_router::*;
use lepu::{Chapter, Content, Epub, Style, Text, TextKind};

use std::{cell::RefCell, rc::Rc};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

type BookResource = Resource<Option<web_sys::File>, Option<Rc<RefCell<Epub>>>>;

#[component]
fn App() -> impl IntoView {
    let (source, set_source) = create_signal(None);
    let res = create_local_resource(
        move || source.get(),
        |file: Option<web_sys::File>| async move {
            use web_sys::js_sys::Uint8Array;
            let file = file?;
            let promise = file.array_buffer();
            let future = wasm_bindgen_futures::JsFuture::from(promise);
            let res = future.await.ok()?;
            let buf = Uint8Array::new(&res).to_vec();
            let epub = Epub::from_vec(buf).ok()?;
            Some(Rc::new(RefCell::new(epub)))
        },
    );

    let (page, set_page) = create_signal(0usize);

    provide_context(page);
    provide_context(set_page);
    provide_context(res);

    let main_view = move || {
        let book_view = move || matches!(res.get(), Some(Some(_)));
        let upload_view = move || {
            view! { <Upload file=set_source /> }
        };
        view! {
            <Show when=book_view fallback=upload_view>
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
        let inner = entries
            .map(|e: &Chapter| {
                let sublist = e.has_children().then(|| make_list(e.children()));
                let idx = format!("{}", e.index_in_spine());
                let name = e.name().to_owned();
                view! {
                    <li><A href=idx class="text-sky-300">{name}</A>
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
                "pt-8 pb-6 text-left font-sans font-bold text-4xl tracking-tight leading-none",
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
    let set_page = expect_context::<WriteSignal<usize>>();
    let book = expect_context::<BookResource>();
    let page = move || match params.with(|p| p.as_ref().map(|p| p.idx).ok().flatten()) {
        Some(page) => page,
        None => {
            // HACK: trailing slash makes this route match
            (use_navigate())("/", Default::default());
            0
        }
    };

    create_effect(move |_| set_page.set(page()));

    let text = move || {
        let mut out: Vec<View> = Vec::new();

        let book = book.get().unwrap().unwrap();
        book.borrow_mut()
            .traverse_chapter(page(), |content, _| match content {
                Content::Textual(tc) => {
                    out.push(convert(&tc));
                }
                Content::Image => {}
            })
            .unwrap();

        Some(out.into_iter().collect_view())
    };

    let navigate = use_navigate();
    let handle = window_event_listener(ev::keyup, move |ev: ev::KeyboardEvent| {
        if ev.alt_key() || ev.shift_key() || ev.meta_key() || ev.ctrl_key() {
            return;
        }
        match &*ev.key() {
            "ArrowLeft" => {
                let id = page();
                if id == 0 {
                    return;
                };
                navigate(&format!("/{}", id - 1), Default::default());
            }
            "ArrowRight" => {
                let id = page();
                let max_id = book.get().unwrap().unwrap().borrow().document_count();
                if id + 1 >= max_id {
                    return;
                };
                navigate(&format!("/{}", id + 1), Default::default());
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
