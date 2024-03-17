use leptos::*;
use leptos_router::A;

use crate::{
    book::{self, Book},
    nav_state::NavState,
};

#[component]
pub fn NavBar(book: ReadSignal<Book>) -> impl IntoView {
    let book_exists = move || matches!(book.get(), Some(_));
    let page = expect_context::<ReadSignal<usize>>();
    let nav_state = expect_context::<ReadSignal<NavState>>();

    view! {
        <nav>
            <div class="flex justify-between px-1 text-sm md:text-base">
                <div>
                    <Show when=book_exists>
                        <button class="hover:text-sky-500" on:click=move |_| book::unload()>"✕"</button>
                    </Show>
                </div>
                <ul class="flex space-x-6 md:space-x-10">
                    <Show when=book_exists fallback=move || view! {
                        <li><span class:underline=move || nav_state.get() == NavState::Upload><A class="hover:text-sky-500" href="">load</A></span></li> }>
                        <li><span class:underline=move || nav_state.get() == NavState::Read><A class="hover:text-sky-500" href={move || format!("{}", page.get())}>read</A></span></li>
                        <li><span class:underline=move || nav_state.get() == NavState::Toc><A class="hover:text-sky-500" href="">table of contents</A></span></li>
                    </Show>
                    <li><span class:underline=move || nav_state.get() == NavState::Settings><A class="hover:text-sky-500" href="settings">settings</A></span></li>
                </ul>
            </div>
        </nav>
    }
}
