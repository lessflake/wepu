use leptos::*;
use leptos_router::*;

use crate::{
    book::{self, Book},
    components::{Content, NavBar, Settings, Toc, Upload},
    config, marks, nav_state, position,
};

// local storage usage (non-normative)
// "b" => base64 encoded epub (most recently loaded book)
// "{book identifier}" => "{page}:{para}" (current position)
// "c" => "{true|false}:{true|false}" (config fields in order)

fn set_title(title: &str) {
    if !title.is_empty() {
        document().set_title(&format!("{title} | wepu"));
    } else {
        document().set_title("wepu");
    }
}

#[component]
pub fn App() -> impl IntoView {
    config::init();
    book::init();
    nav_state::init();
    position::init();
    marks::init();

    let book = expect_context::<ReadSignal<Book>>();

    create_effect(move |_| {
        if let Some(book) = book.get() {
            set_title(book.title());
        }
    });

    let book_exists = move || book.get().is_some();
    let main_view = move || {
        view! {
            <Show when=book_exists fallback=Upload>
                <Outlet/>
            </Show>
        }
    };

    view! {
        <Router base="/wepu">
            <main>
                <div class="flex flex-col max-w-screen-sm md:max-w-screen-md
                            min-h-screen mx-auto px-2 pb-2 pt-2 md:pt-10
                            text-base sm:text-lg md:text-2xl">
                    <NavBar book=book />
                    <Routes base="/wepu".to_string()>
                        <Route path="/" view=main_view>
                            <Route path="" view=Toc />
                            <Route path=":idx" view=Content />
                        </Route>
                        <Route path="settings" view=Settings />
                    </Routes>
                </div>
            </main>
        </Router>
    }
}
