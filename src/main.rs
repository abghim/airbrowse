use anyhow::Result;
use slint::{ComponentHandle, Timer, TimerMode};
use url::Url;
use wry::{
    dpi::{LogicalSize, Position},
    Rect, WebView, WebViewBuilder, Error as WryError,
};

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

	let chrome_h:f64 = 40.0;

	use std::{cell::RefCell, rc::Rc};
	let webview_cell: Rc<RefCell<Option<WebView>>> = Rc::new(RefCell::new(None));

	let init_timer: &'static Timer = Box::leak(Box::new(Timer::default()));
    let resize_timer: &'static Timer = Box::leak(Box::new(Timer::default()));
    {
        let weak = app.as_weak();
        let webview_cell = webview_cell.clone();

        println!("start initialization of webView");
        init_timer.start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(25),
            move || {
                let Some(app) = weak.upgrade() else { return; };
                let win = app.window();

                let sf = win.scale_factor() as f64;
                let phys = win.size();
                if phys.width == 0 || phys.height == 0 || sf == 0.0 {
                    return;
                }

                let w = phys.width as f64 / sf;
                let h = phys.height as f64 / sf - chrome_h;

                let mut start = app.get_url().to_string();
                if start.trim().is_empty() {
                    start = "https://abghim.github.io".to_string();
                    app.set_url(start.clone().into());
                }
                let start = canonicalize(&start);

                let handle = win.window_handle();

                match WebViewBuilder::new()
                    .with_url(&start)
                    .with_bounds(Rect {
                        position: Position::Logical((0.0, chrome_h).into()),
                        size: LogicalSize::new(w, h).into(),
                    })
                    .build_as_child(&handle)
                {
                    Ok(wv) => {
                        println!("Webview created");
                        *webview_cell.borrow_mut() = Some(wv);
                        init_timer.stop();
                    }
                    Err(WryError::WindowHandleError(_)) => {
                    }
                    Err(e) => {
                        eprintln!("[init] build_as_child failed (retrying): {e}");
                    }
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

        resize_timer.start(
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
