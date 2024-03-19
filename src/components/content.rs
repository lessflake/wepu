use std::{cell::RefCell, rc::Rc};

use leptos::*;
use leptos_router::*;

use crate::{
    book::Book,
    content::chapter_to_html,
    input::{Action, Handler},
    nav_state::NavState,
    position::Tracker,
    set_nav_state,
};

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

    let page = expect_context::<ReadSignal<usize>>();
    let set_page = expect_context::<WriteSignal<usize>>();
    let book = expect_context::<ReadSignal<Book>>();
    let param_page = move || {
        params
            .with(|p| p.as_ref().map(|p| p.idx).ok().flatten())
            .unwrap_or(0)
    };
    create_effect(move |_| set_page.set(param_page()));

    let tracker_ = Rc::new(Tracker::init());
    let tracker = tracker_.clone();
    let handler = RefCell::new(Handler::new());

    let handle = window_event_listener(ev::keyup, move |ev: ev::KeyboardEvent| {
        if ev.alt_key() || ev.shift_key() || ev.meta_key() || ev.ctrl_key() {
            return;
        }
        let Some(action) = handler.borrow_mut().handle(&*ev.key()) else {
            return;
        };

        match action {
            Action::NextPage => tracker.move_to_next_page(),
            Action::PreviousPage => tracker.move_to_previous_page(),
            Action::SetMark(c) => tracker.set_mark(c),
            Action::FollowMark(c) => tracker.follow_mark(c),
            Action::Leave => (use_navigate())("/", Default::default()),
        }
    });

    on_cleanup(move || {
        handle.remove();
    });

    let tracker = tracker_.clone();
    let move_next = move || tracker.move_to_next_page();
    let tracker = tracker_.clone();
    let move_previous = move || tracker.move_to_previous_page();

    let tracker = tracker_.clone();
    view! {
        {move || {
            let tracker = tracker.clone();
            view! {
                <div class="sm:text-justify font-serif font-light space-y-3 md:space-y-5 mt-8">
                    {chapter_to_html(tracker, book, page)}
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
