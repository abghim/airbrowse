# airBrowse: minimal WebKit browser for Mac

`airBrowse` is an intentionally minimal browser utilizing MacOS native WebKit. It is written entirely in Rust using the Slint user interface framework and the `wry` crate that allows access to WebKit as a child window.

As of v0.1.0, the browser is ripe with bugs, including but not limited to:
- Enter to submit form does not work in some pages like DuckDuckGo
- No cmd-C, cmd-V nor other editing commands
- Slow, inefficient, and unfriendly search/url bar; no tab system, with mostly empty chrome
