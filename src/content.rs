use std::rc::Rc;

use base64::prelude::*;
use leptos::*;
use lepu::{Content, Style, Text, TextKind};

use crate::{book::Book, position};

pub fn chapter_to_html(
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
