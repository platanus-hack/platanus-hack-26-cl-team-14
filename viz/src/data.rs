//! Data files: from disk in the native app, from the page in the browser.
use std::io::{self, Read};
use std::path::Path;

#[cfg(not(target_family = "wasm"))]
pub fn open(path: impl AsRef<Path>) -> io::Result<impl Read> {
    std::fs::File::open(path)
}

#[cfg(not(target_family = "wasm"))]
pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    std::fs::read(path)
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    /// The page downloads the files before it starts the viewer. A missing file is `undefined`.
    #[wasm_bindgen(js_name = santiagoFile)]
    fn file(path: &str) -> Option<Vec<u8>>;
}

#[cfg(target_family = "wasm")]
pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let path = path.as_ref().to_str().unwrap();
    file(path).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.to_owned()))
}

#[cfg(target_family = "wasm")]
pub fn open(path: impl AsRef<Path>) -> io::Result<impl Read> {
    read(path).map(io::Cursor::new)
}
