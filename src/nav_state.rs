use leptos::*;

#[derive(Clone, Copy, PartialEq)]
pub enum NavState {
    Upload,
    Read,
    TableOfContents,
    Settings,
}

pub fn set_nav_state(state: NavState) {
    let set_nav_state = expect_context::<WriteSignal<NavState>>();
    set_nav_state.set(state);
}
