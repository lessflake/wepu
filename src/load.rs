use leptos::*;

use crate::nav_state::{set_nav_state, NavState};

#[component]
pub fn Upload() -> impl IntoView {
    set_nav_state(NavState::Upload);

    let file = expect_context::<WriteSignal<Option<web_sys::File>>>();
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
