use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod debug_info;
mod getter;
mod utils;

use debug_info::{derive_section_debug_info_impl, derive_template_debug_info_impl};
use getter::derive_getter_impl;

/// ゲッター導出マクロ
///
/// ```text
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
/// ```text
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

/// 節デバッグ情報出力導出マクロ
///
/// ```text
/// #[derive(SectionDebugInfo)]
/// #[section(number=1, name="識別節")]
/// pub struct Section1 {
///     #[debug_info(name="節の長さ", fmt="0x{:04X}")]
///     section_bytes: usize,
///     #[debug_info(name="作成中枢の識別")]
///     center: u16,
/// }
///
/// #[derive(SectionDebugInfo)]
/// #[section(number=3, name="格子系定義節")]
/// pub struct Section3<T> {
///     #[debug_info(name="節の長さ", fmt="0x{:04X}")]
///     section_bytes: usize,
///     #[debug_info(name="格子系定義の出典")]
///     source_of_grid_definition: u8,
///     #[debug_template]
///     template3: T,
/// }
/// ```
///
/// 上記から次を導出する。
///
/// ```text
/// impl Section0 {
///     pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
///     where
///         W: std::io::Write,
/// {
///     writeln!(writer, "第1節:識別節")?;
///     writeln!(writer, "    節の長さ: 0x{04X}", self.section_bytes)?;
///     writeln!(writer, "    作成中枢の識別: {}", self.center)?;
///
///     Ok(())
/// }
///
/// impl<T> Section3<T> {
///     pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
///     where
///         T: DebugTemplate<W>,
///         W: std::io::Write,
/// {
///     writeln!(writer, "第3節:格子系定義節")?;
///     writeln!(writer, "    節の長さ: {}", self.section_bytes)?;
///     writeln!(writer, "    格子系定義の出典: {}", self.source_of_grid_definition())?;
///     self.template3.debug_info(writer)?;
///
///    Ok(())
/// }
/// ```
#[proc_macro_derive(SectionDebugInfo, attributes(section, debug_info, debug_template))]
pub fn derive_section_debug_info(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_section_debug_info_impl(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// 節デバッグ情報出力導出マクロ
///
/// ```text
/// #[derive(TemplateDebugInfo)]
/// pub struct Template3_0 {
///     #[debug_info(name="地球の形状", fmt="{}")]
///     shape_of_earth: u8,
///     /// 地球回転楕円体の長軸の尺度因子
///     #[debug_info(name="地球回転楕円体の長軸の尺度因子")]
///     scale_factor_of_earth_major_axis: u8,
/// }
/// ```
///
/// 上記構造体から次を導出する。
///
/// ```text
/// impl<W> DebugTemplate<W> for Template3_0 {
///     fn debug_info(&self, writer: &mut W) -> std::io::Result<()> {
///         write!(writer, "    地球の形状: {}", self.shape_of_earth)?;
///         write!(writer, "    地球回転楕円体の長軸の尺度因子: {}", self.scale_factor_of_earth_major_axis)?;
///
///         Ok(())
///     }
/// }
/// ```
#[proc_macro_derive(TemplateDebugInfo, attributes(debug_info))]
pub fn derive_template_debug_info(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_template_debug_info_impl(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}
