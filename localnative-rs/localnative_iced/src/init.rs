//! Native-messaging host manifest installation for the browser extension.
//!
//! The implementation lives in the shared `localnative_hostinstall` crate;
//! this module keeps the async shape the app was written against.

/// Install both browsers' manifests (and, on Linux, the host binary copy).
/// `allowed_origins` overrides the Chrome extension ID list (used when
/// loading the extension unpacked during development).
pub async fn init_all(allowed_origins: Option<Vec<String>>) {
    localnative_hostinstall::WebKind::install_all(allowed_origins);
}
