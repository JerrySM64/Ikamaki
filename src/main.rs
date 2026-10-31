#[cfg(not(target_arch = "wasm32"))]
mod cli;
mod core;
mod gui;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let raw: Vec<String> = std::env::args().collect();
    let gui_requested =
        raw.len() <= 1 || raw.iter().skip(1).any(|arg| arg.as_str() == "--gui");

    if gui_requested {
        gui::run();
    } else {
        cli::run();
    }
}

// When compiling to web (wasm32-unknown-unknown):
#[cfg(target_arch = "wasm32")]
fn main() {
    gui::run();
}
