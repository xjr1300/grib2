use macros::SectionDebugInfo;

#[derive(SectionDebugInfo)]
#[section(number = 3, name = "格子系定義節")]
pub struct Section3<T> {
    #[debug_info(name = "節の長さ", fmt = "0x{:04X}")]
    section_bytes: usize,
    #[debug_info(name = "格子系定義の出典")]
    source_of_grid_definition: u8,
    #[debug_template]
    template3: T,
}

pub trait DebugTemplate<W> {
    /// テンプレートのデバッグ情報を出力する。
    ///
    /// # 引数
    ///
    /// * `writer` - 出力先
    fn debug_info(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write;
}

#[derive(SectionDebugInfo)]
#[section(number = 0, name = "地域使用節")]
pub struct Section2;
