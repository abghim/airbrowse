use anyhow::Result;
use slint::ComponentHandle;
use url::Url;
use wry::{Rect, WebView, WebViewBuilder, dpi::{LogicalSize, Position}};

slint::include_modules!();

// struct ParentHandle<'a> {
//     window: WindowHandle<'a>,
//     display: DisplayHandle<'a>,
// }

// impl<'a> HasWindowHandle for ParentHandle<'a> {
//     fn window_handle(&self) -> Result<WindowHandle, raw_window_handle::HandleError> {
//         Ok(self.window.clone())
//     }
// } impl<'a> HasDisplayHandle for ParentHandle<'a> {
//     fn display_handle(&self) -> Result<DisplayHandle, raw_window_handle::HandleError> {
//         Ok(self.display.clone())
//     }
// }


fn canonicalize(input: &str) -> String {
    let s = input.trim();
    if s.is_empty() {
        return "about:blank".into();
    }
    if Url::parse(s).is_ok() {
        return s.to_string();
    }
    if s.contains('.') && !s.contains(' ') {
        return format!("https://{s}");
    }
    format!("https://duckduckgo.com/?q={}", urlencoding::encode(s))
}

fn main() -> Result<()> {
	unsafe {
		std::env::set_var("SLINT_BACKEND", "winit-femtovg");
	}

	let app = Browser::new()?;
	println!("Made app");

	app.show()?;

	use std::{cell::RefCell, rc::Rc};
	let webview_cell: Rc<RefCell<Option<WebView>>> = Rc::new(RefCell::new(None));

    // CHANGED: defer WebView creation to "soon after loop start" for macOS robustness.
    {
        let weak = app.as_weak();
        let webview_cell = webview_cell.clone();

        let init = slint::Timer::default();
        println!("start initialization of webView");
        init.start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(0),
            move || {
                if let Some(app) = weak.upgrade() {
                    let window = app.window();

                    // Slint's handle already implements HasWindowHandle + HasDisplayHandle.
                    // No wrapper type needed.
                    let handle = window.window_handle();

                    // Compute logical (point) size for wry (Retina safe).
                    let sf = window.scale_factor() as f64;
                    let phys = window.size(); // physical pixels
                    let chrome_h = 40.0;      // keep in sync with your Slint top bar height
                    let logical = LogicalSize::new(
                        phys.width as f64 / sf,
                        phys.height as f64 / sf - chrome_h,
                    );

                    // Build a child WKWebView positioned under the 40px bar.
                    let wv = WebViewBuilder::new()
                        // .with_url(&app.get_url())
                        .with_html("<html><body><h1>It works</h1></body></html>")
                        .with_bounds(Rect {
                            position: Position::Logical((0.0, chrome_h).into()),
                            size: logical.into(),
                        })
                        .build_as_child(&handle)
                        .expect("build_as_child failed");


                    let _ = wv.focus();

                    println!("Webview created");

                    *webview_cell.borrow_mut() = Some(wv);
                }
            },
        );
    }

    {
    	println!("Go callback definition");
		let weak = app.as_weak();
	    let webview_cell = webview_cell.clone();
		app.on_go(
			move |typed| {
				let target = canonicalize(&typed);
    			if let Some(wv) = webview_cell.borrow().as_ref() {
                	let _ = wv.load_url(&target);
            	}
				if let Some(appl) = weak.upgrade() {
					appl.set_url(target.into());
				}
			}
		);
    }

    {
    	println!("Resize timer started");
        let weak = app.as_weak();
        let webview_cell = webview_cell.clone();
        let resize = slint::Timer::default();
        resize.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(100),
            move || {
                if let (Some(app), Some(wv)) = (weak.upgrade(), webview_cell.borrow().as_ref()) {
                    let w = app.window();
                    let sf = w.scale_factor() as f64;
                    let phys = w.size();
                    let chrome_h = 40.0;
                    let width = phys.width as f64 / sf;
                    let height = phys.height as f64 / sf - chrome_h;
                    let _ = wv.set_bounds(Rect {
                        position: Position::Logical((0.0, chrome_h).into()),
                        size: LogicalSize::new(width, height).into(),
                    });
                }
            },
        );
    }

    println!("Starting event loop");

    slint::run_event_loop()?;
	Ok(())
}
