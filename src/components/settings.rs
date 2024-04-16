use leptos::*;

use crate::{
    config,
    nav_state::{set_nav_state, NavState},
};

#[component]
pub fn Settings() -> impl IntoView {
    set_nav_state(NavState::Settings);

    let clear_storage = || {
        if let Ok(Some(storage)) = leptos::window().local_storage() {
            let _ = storage.clear();
        }
    };

    let config = config::get();
    let config_ = config.clone();

    let save_position = move |ev| {
        config_.borrow_mut().save_position = event_target_checked(&ev);
        config_.borrow().save();
    };
    let config_ = config.clone();
    let cache_book = move |ev| {
        let checked = event_target_checked(&ev);
        config_.borrow_mut().cache_book = checked;
        config_.borrow().save();
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
            <div><button class="bg-sepia-dark text-sepia-light active:text-sepia-light dark:bg-zinc-200 dark:text-zinc-800 mt-2 active:bg-sky-500 dark:active:text-zinc-200 rounded-lg px-3 py-1" on:click=move |_| clear_storage()>Clear data</button></div>
        </div>
    }
}
