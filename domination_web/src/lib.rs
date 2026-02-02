pub use include_dir::*;


static SVELTE_DIST: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/svelte/dist");

pub fn web_files() -> &'static Dir<'static> {
    &SVELTE_DIST
}

