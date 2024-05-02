#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod wrap_app;
pub use app::TodoApp;
pub use wrap_app::WrapApp;
