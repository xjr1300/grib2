use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod getter;
mod template_getter;
mod utils;

use getter::derive_getter_impl;
use template_getter::derive_template_getter_impl;

/// ゲッター導出マクロ
///
/// ```rust
/// #[derive(Getter)]
/// pub struct Foo {
///     #[getter(ret="val")]
///     a: i32,
///     #[getter(ret="ref")]
///     b: PathBuf,
///     #[getter(ret="ref", ty="&str")]
///     c: String,
/// }
/// ```
///
/// 上記構造体から次を導出する。
///
/// ```rust
/// impl Foo {
///     pub fn a(&self) -> i32 {
///         self.a
///     }
///     pub fn b(&self) -> &PathBuf {
///          &self.b
///     }
///     pub fn c(&self) -> &str {
///        &self.c
///     }
/// }
/// ```
#[proc_macro_derive(Getter, attributes(getter))]
pub fn derive_getter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_getter_impl(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// テンプレートゲッター導出マクロ
///
/// ```rust
/// struct Section2<T2> {
///     template2: T2,
/// }
///
/// #[derive(TemplateGetter(section="Section2", member="template2"))]
/// struct Template2 {
///     #[getter(ret="val")]
///     a: i32,
///     #[getter(ret="ref")]
///     b: PathBuf,
///     #[getter(ret="ref", ty="&str")]
///     c: String,
/// }
/// ```
///
/// 上記構造体から次を導出する。
///
/// ```rust
/// impl Section2<Template2> {
///     pub fn a(&self) -> i32 {
///         self.template2.a
///     }
///
///     pub fn b(&self) -> &PathBuf {
///         &self.template2.b
///     }
///
///     pub fn c(&self) -> &str {
///         &self.template2.c
///     }
/// }
/// ```
#[proc_macro_derive(TemplateGetter, attributes(template_getter, getter))]
pub fn derive_template_getter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_template_getter_impl(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}
