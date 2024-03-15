use leptos::*;
use leptos_router::*;

use crate::{
    book::{self, Book},
    components::{Content, NavBar, Settings, Toc, Upload},
    config, marks, position,
};

// local storage usage (non-normative)
// "b" => base64 encoded epub (most recently loaded book)
// "{book identifier}" => "{page}:{para}" (current position)
// "{book identifier}-route" => "nav" | "content" (current route)
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
    create_effect(move |_| {
        (use_navigate())("", Default::default());
    });

    config::init();
    book::init();
    position::init();
    marks::init();

    let book = expect_context::<ReadSignal<Book>>();

    create_effect(move |_| {
        if let Some(book) = book.get() {
            let epub = book.borrow();
            set_title(epub.title());
        }
    });

    let book_exists = move || matches!(book.get(), Some(_));

    let upload_view = move || {
        view! { <Upload /> }
    };

    let main_view = move || {
        view! {
            <Show when=book_exists fallback=upload_view>
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
