use leptos::*;
use leptos_router::A;
use lepu::Chapter;

use crate::{book::Book, nav_state::NavState, set_nav_state};

fn make_list<'a>(entries: impl Iterator<Item = &'a Chapter>) -> leptos::View {
    let page = expect_context::<ReadSignal<usize>>();
    let inner = entries
        .map(|e: &Chapter| {
            let sublist = e.has_children().then(|| make_list(e.children()));
            let idx = e.index_in_spine().to_string();
            let name = e.name().to_owned();
            let class = if e.index_in_spine() == page.get() {
                "pb-2 underline"
            } else {
                "pb-2"
            };
            view! {
                <li><div class=class><A href=idx class="hover:text-sky-500">{name}</A></div>
                    {sublist}
                </li>
            }
        })
        .collect_view();

    html::ul().classes("ml-5").child(inner).into_view()
}

#[component]
pub fn Toc() -> impl IntoView {
    set_nav_state(NavState::Toc);

    let book = expect_context::<ReadSignal<Book>>();
    view! {
        { move || {
            let book = book.get().unwrap();
            view! {
                <div class="mt-8 mb-10 text-left font-bold text-2xl md:text-4xl tracking-tight leading-none">
                    <h1>{book.title().to_owned()}</h1>
                </div>
                <div class="text-justify font-serif tracking-tight leading-tight">
                    {make_list(book.chapters())}
                </div>
            }
        }}
    }
}
