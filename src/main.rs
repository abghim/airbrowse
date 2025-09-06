use anyhow::Result;
use slint::ComponentHandle;
use url::Url;
use wry::{Rect, WebView, WebViewBuilder, dpi::LogicalSize};

slint::include_modules!();
