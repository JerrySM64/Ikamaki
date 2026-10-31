mod app;

pub use app::IkamakiApp;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    #[cfg(target_os = "linux")]
    prefer_x11_if_wayland_unavailable();

    if let Err(err) = IkamakiApp::new().and_then(|app| app.run()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

#[cfg(target_os = "linux")]
fn prefer_x11_if_wayland_unavailable() {
    use std::env;

    if env::var_os("WINIT_UNIX_BACKEND").is_some() {
        return;
    }

    let display = env::var("WAYLAND_DISPLAY").ok().filter(|v| !v.is_empty());
    let socket_fd = env::var("WAYLAND_SOCKET").ok().filter(|v| !v.is_empty());
    if display.is_none() && socket_fd.is_none() {
        return;
    }
    if !env::var("DISPLAY").is_ok_and(|v| !v.is_empty()) {
        return;
    }

    let lib_ok = wayland_client_lib_loads();
    let socket_ok = socket_fd.is_some() || display.as_deref().is_some_and(wayland_socket_exists);
    if lib_ok && socket_ok {
        return;
    }

    eprintln!("Wayland is not usable here; falling back to the X11 (XWayland) backend");
    unsafe {
        env::remove_var("WAYLAND_DISPLAY");
        env::remove_var("WAYLAND_SOCKET");
        env::set_var("WINIT_UNIX_BACKEND", "x11");
    }
}

#[cfg(target_os = "linux")]
fn wayland_client_lib_loads() -> bool {
    unsafe { libloading::Library::new("libwayland-client.so.0").is_ok() }
}

#[cfg(target_os = "linux")]
fn wayland_socket_exists(display: &str) -> bool {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR");
    display.split(',').any(|name| {
        let path = std::path::Path::new(name);
        if path.is_absolute() {
            path.exists()
        } else if let Some(runtime) = &runtime {
            std::path::Path::new(runtime).join(name).exists()
        } else {
            false
        }
    })
}

// When compiling to web (wasm32-unknown-unknown):
#[cfg(target_arch = "wasm32")]
pub fn run() {
    console_error_panic_hook::set_once();
    if let Err(err) = IkamakiApp::new().and_then(|app| app.run()) {
        panic!("Failed to start Ikamaki: {err}");
    }
}
