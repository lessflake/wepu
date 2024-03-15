use leptos::mount_to_body;
use wepu::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
