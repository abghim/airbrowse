use anyhow::Result;
use slint::ComponentHandle;
use url::Url;
use wry::{Rect, WebView, WebViewBuilder, dpi::LogicalSize};

slint::include_modules!();

fn canonicalize(input: &str) -> String {
    let s = input.trim();
    if s.is_empty() {
        return "about:blank".into();
    }
    // If it parses as an absolute URL, use it.
    if Url::parse(s).is_ok() {
        return s.to_string();
    }
    // If it looks like a host without scheme, prefix https://
    if s.contains('.') && !s.contains(' ') {
        return format!("https://{s}");
    }
    // Otherwise treat as a search query (DuckDuckGo)
    let q = urlencoding::encode(s);
    format!("https://duckduckgo.com/?q={q}")
}

fn build_child_webview(
    window: &slint::Window,
    start_url: &str,
    on_hotkey: impl Fn() + 'static + Send,
) -> Result<WebView> {
    // 1) Native handle from Slint → required by wry’s child API.
    let handle = window.window_handle(); // needs raw-window-handle-06
    // 2) Size: on macOS, webview auto-resizes with its parent, but we set an initial bound.
    let size = window.size(); // physical size
    let logical_size = LogicalSize::new(size.width as f64, size.height as f64);

    // 3) Build the WKWebView as a child NSView living inside the Slint window.
    //    We also register an IPC handler and an init script so ⌥Enter works even
    //    when the page itself has keyboard focus.
    let webview = WebViewBuilder::new()
        .with_url(start_url)
        .with_initialization_script(
            r#"
            // Make ⌥Enter open the address UI even if focus is inside the page.
            document.addEventListener('keydown', (e) => {
              if (e.altKey && e.key === 'Enter') {
                window.ipc.postMessage('hotkey:address');
              }
            }, {capture:true});
            "#,
        )
        .with_ipc_handler(move |req| {
            if req.body() == "hotkey:address" {
                on_hotkey();
            }
        })
        .with_bounds(Rect {
            position: (0.0, 0.0).into(),
            size: logical_size.into(),
        })
        .build_as_child(&handle)?;
    Ok(webview)
}

fn main() -> Result<()> {
    let app = Browser::new()?;

    // Create the child WKWebView and wire the hotkey callback back into Slint.
    let weak = app.as_weak();
    let window = app.window();
    let start_url = app.get_url();

    let _webview = build_child_webview(&window, &start_url, move || {
        if let Some(app) = weak.upgrade() {
            // Ask the UI to show the overlay and focus the address bar.
            app.invoke_show_overlay_from_host();
        }
    })?;

    // When user hits Enter in the overlay, navigate the webview.
    let weak_webview = std::sync::Arc::new(std::sync::Mutex::new(None::<WebView>));
    // store real webview (created above)
    {
        let wv = _webview.clone();
        *weak_webview.lock().unwrap() = Some(wv);
    }

    let wv_for_go = weak_webview.clone();
    app.on_go(move |text| {
        if let Some(ref wv) = *wv_for_go.lock().unwrap() {
            let target = canonicalize(&text);
            let _ = wv.load_url(&target);
        }
    });

    app.run()?;
    Ok(())
}
