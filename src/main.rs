use anyhow::Result;
use slint::ComponentHandle;
use url::Url;
use wry::{Rect, WebView, WebViewBuilder, dpi::LogicalSize};

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
	let window = app.window();
	let weak = app.as_weak();
	let handle = window.window_handle();


	app.show()?;

	let sf = window.scale_factor() as f64;
	let chrome_h: f64 = 40.0;
	let _size = window.size();
	let initial_size = LogicalSize::new((_size.width as f64)/sf, (_size.height as f64)/sf - chrome_h);


	let mut webview: WebView = WebViewBuilder::new()
		.with_url(&app.get_url())
		.with_bounds(Rect { position: wry::dpi::Position::Logical((0.0, chrome_h).into()), size: initial_size.into() })
		.build_as_child(&handle)?;

	// let window_handle = window.window_handle()?;
	// let display_handle = window.display_handle()?;
	// let parent = ParentHandle { window: window_handle, display: display_handle };

	// let mut webview: WebView = WebViewBuilder::new()
	// 	.with_url(&app.get_url())
	// 	.with_bounds(Rect { position: wry::dpi::Position::Logical((0.0, chrome_h).into()), size: initial_size.into() })
	// 	.build_as_child(&parent)?;

	app.on_go(
		move |typed| {
			let target = canonicalize(&typed);
			let _ = webview.load_url(&target);
			if let Some(appl) = weak.upgrade() {
				appl.set_url(target.into());
			}
		}
	);

    slint::run_event_loop()?;
	Ok(())
}
