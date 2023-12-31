use macros::{SectionDebugInfo, TemplateDebugInfo};

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

#[derive(TemplateDebugInfo)]
pub struct Template7_200 {
    #[debug_info(name = "ランレングス圧縮符号列の開始位置", fmt = "0x{:08X}")]
    run_length_position: usize,
    /// ランレングス圧縮符号列のバイト数
    #[debug_info(name = "ランレングス圧縮符号のバイト数")]
    run_length_bytes: usize,
}

#[derive(TemplateDebugInfo)]
pub struct Template5_200 {
    #[debug_info(name = "今回の圧縮に用いたレベルの最大値")]
    max_level_value: u16,
    #[debug_info(name = "データの取り得るレベルの最大値")]
    number_of_level_values: u16,
    #[debug_info(name = "データ代表値の尺度因子")]
    decimal_scale_factor: u8,
    #[debug_info(
        data_type = "serial",
        name = "レベルmに対応するデータ代表値",
        header = "レベル{}",
        start = 1,
        fmt = "{}"
    )]
    level_values: Vec<u16>,
}
