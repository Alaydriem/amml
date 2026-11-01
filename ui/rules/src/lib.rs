//! Source scanners that the UI crates run in their tests to enforce the UI and design rules of the amml workspace
//! CLAUDE.md: no design literals, no copies of an amml-ui widget, and no `widgets/` or `theme/` folder in an app crate.

#![forbid(unsafe_code)]

pub mod folder;
pub mod literal;
pub mod source;
pub mod widget_name;
