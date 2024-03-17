use leptos::*;

#[derive(Clone, Copy, PartialEq)]
pub enum NavState {
    Upload,
    Read,
    Toc,
    Settings,
}

pub fn init() {
    let (nav_state, set_nav_state) = create_signal(NavState::Upload);
    provide_context(nav_state);
    provide_context(set_nav_state);
}

pub fn set_nav_state(state: NavState) {
    let set_nav_state = expect_context::<WriteSignal<NavState>>();
    set_nav_state.set(state);
}
