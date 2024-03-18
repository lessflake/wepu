use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use base64::prelude::*;
use leptos::*;
use leptos_router::*;
use lepu::{Content, Style, Text, TextKind};

use crate::{book::Book, input, nav_state::NavState, position, set_nav_state, Marks};

#[derive(Params, Debug, Clone, PartialEq)]
struct ChapterParams {
    idx: Option<usize>,
}

#[component]
pub fn Content() -> impl IntoView {
    let params = use_params::<ChapterParams>();
    if !matches!(params.get_untracked(), Ok(ChapterParams { idx: Some(_) })) {
        (use_navigate())("", Default::default());
    }

    set_nav_state(NavState::Read);

    let set_pos = expect_context::<WriteSignal<BTreeMap<usize, usize>>>();
    let page = expect_context::<ReadSignal<usize>>();
    let set_page = expect_context::<WriteSignal<usize>>();
    let book = expect_context::<ReadSignal<Book>>();
    let param_page = move || {
        params
            .with(|p| p.as_ref().map(|p| p.idx).ok().flatten())
            .unwrap_or(0)
    };
    create_effect(move |_| set_page.set(param_page()));

    let tracker = Rc::new(position::init_tracking());

    let tracker_ = tracker.clone();
    let navigate = use_navigate();
    let handler = RefCell::new(input::Handler::new());
    let marks = expect_context::<Marks>();

    let move_next = move || {
        let id = param_page();
        let max_id = book.get().unwrap().document_count();
        if id + 1 < max_id {
            navigate(&(id + 1).to_string(), Default::default());
        }
    };

    let navigate = use_navigate();
    let move_previous = move || {
        let id = param_page();
        if id > 0 {
            navigate(&(id - 1).to_string(), Default::default());
        }
    };

    let navigate = use_navigate();
    let move_next_ = move_next.clone();
    let move_previous_ = move_previous.clone();
    let handle = window_event_listener(ev::keyup, move |ev: ev::KeyboardEvent| {
        if ev.alt_key() || ev.shift_key() || ev.meta_key() || ev.ctrl_key() {
            return;
        }
        let Some(action) = handler.borrow_mut().handle(&*ev.key()) else {
            return;
        };

        use input::Action;
        match action {
            Action::NextPage => move_next_(),
            Action::PreviousPage => move_previous_(),
            Action::SetMark(c) => {
                let Some(para) = tracker_.first_visible() else {
                    return;
                };
                let page = param_page();
                marks.borrow_mut().insert(c, (page, para));
            }
            Action::FollowMark(c) => {
                if let Some((page, para)) = marks.borrow().get(&c).copied() {
                    set_pos.update(move |pos| _ = pos.insert(page, para));
                    navigate(&page.to_string(), Default::default());
                }
            }
            Action::Leave => navigate("/", Default::default()),
        }
    });

    on_cleanup(move || {
        handle.remove();
    });

    view! {
        {move || {
            let tracker = tracker.clone();
            view! {
                <div class="sm:text-justify font-serif font-light space-y-3 md:space-y-5 mt-8">
                    {text(tracker, book, page)}
                </div>
            }
        }}
        <div class="flex justify-center pt-2 md:pt-4 pb-4">
            <div>
                <button class="mt-2 px-3 hover:text-sky-500"
                        on:click=move |_| move_previous()>
                    "←"
                </button>
                <button class="mt-2 px-3 hover:text-sky-500"
                        on:click=move |_| move_next()>
                    "→"
                </button>
            </div>
        </div>
    }
}

fn text(
    tracker: Rc<position::Tracker>,
    book: ReadSignal<Book>,
    page: ReadSignal<usize>,
) -> impl IntoView {
    let mut out: Vec<View> = Vec::new();

    let mut id = 0;
    let book = book.get().unwrap();
    let page = page.get();
    book.traverse_chapter(page, |ctx, content, _| {
        let view = match content {
            Content::Textual(tc) => convert(&tc).into_view(),
            Content::Image(item) => {
                let Ok(data) = ctx.load(&item) else { return };
                let mime = item.mime();
                let mut data_string = format!("data:{mime};base64,");
                BASE64_STANDARD.encode_string(&data, &mut data_string);
                view! {
                    <div class="flex justify-center py-2">
                        <img src=data_string />
                    </div>
                }
                .into_view()
            }
        };
        let tracker = tracker.clone();
        let view = html::div()
            .id(id.to_string())
            .child(view)
            .on_mount(move |node| tracker.track(node, id, page));
        out.push(view.into_view());
        id += 1;
    })
    .unwrap();

    Some(out.into_iter().collect_view())
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
