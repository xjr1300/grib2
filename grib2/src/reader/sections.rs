use std::io::{Read, Seek};

use time::{Date, Month, PrimitiveDateTime, Time};

use super::{FileReader, ReaderError, ReaderResult};
use macros::Getter;

/// 第0節:GRIB版番号
const EDITION_NUMBER: u8 = 2;

/// 第1節:節の長さ（バイト）
const SECTION1_BYTES: u32 = 21;

/// 第3節:格子系定義テンプレート番号
const LAT_LON_GRID_DEFINITION_TEMPLATE_NUMBER: u16 = 0; // 緯度・経度格子

/// 第４節:プロダクト定義テンプレート番号
const RADAR_PRODUCT_DEFINITION_TEMPLATE_NUMBER: u16 = 50008; // レーダーなどに基づく解析プロダクト

/// 第5節:資料表現テンプレート番号
const RUN_LENGTH_DATA_REPRESENTATION_TEMPLATE_NUMBER: u16 = 200; // ランレングス圧縮

/// 第6節:節の長さ（バイト）
const SECTION6_BYTES: u32 = 6;

/// 第8節:終端のマーカー
const SECTION8_END_MARKER: &str = "7777";

/// 第0節:指示節
#[derive(Debug, Clone, Copy, Getter)]
pub struct Section0 {
    /// 資料分野
    #[getter(ret = "val")]
    discipline: u8,
    /// GRIB版番号
    #[getter(ret = "val")]
    edition_number: u8,
    /// GRIB報全体のバイト数
    #[getter(ret = "val")]
    total_length: usize,
}

/// 第1節:識別節
#[derive(Debug, Clone, Copy, Getter)]
pub struct Section1 {
    /// 節の長さ
    #[getter(ret = "val")]
    section_bytes: usize,
    /// 作成中枢の識別
    #[getter(ret = "val")]
    center: u16,
    /// 作成副中枢
    #[getter(ret = "val")]
    sub_center: u16,
    /// GRIBマスター表バージョン番号
    #[getter(ret = "val")]
    table_version: u8,
    /// GRIB地域表バージョン番号
    #[getter(ret = "val")]
    local_table_version: u8,
    /// 参照時刻の意味
    #[getter(ret = "val")]
    significance_of_reference_time: u8,
    /// 資料の参照時刻
    #[getter(ret = "val")]
    referenced_at: PrimitiveDateTime,
    /// 作成ステータス
    #[getter(ret = "val")]
    production_status_of_processed_data: u8,
    /// 資料の種類
    #[getter(ret = "val")]
    type_of_processed_data: u8,
}

/// 第2節:地域使用節
#[derive(Debug, Clone, Copy)]
pub struct Section2;

/// 第3節:格子系定義節
#[derive(Debug, Clone, Copy, Getter)]
pub struct Section3<T3> {
    /// 節の長さ
    #[getter(ret = "val")]
    section_bytes: usize,
    /// 格子系定義の出典
    #[getter(ret = "val")]
    source_of_grid_definition: u8,
    /// 第3節に記録されている資料点数
    #[getter(ret = "val")]
    number_of_data_points: u32,
    /// 格子点数を定義するリストのオクテット数
    #[getter(ret = "val")]
    number_of_octets_for_number_of_points: u8,
    /// 格子点数を定義するリストの説明
    #[getter(ret = "val")]
    interpretation_of_number_of_points: u8,
    /// 格子系定義テンプレート番号
    #[getter(ret = "val")]
    grid_definition_template_number: u16,
    /// テンプレート3
    template3: T3,
}

/// テンプレート3.0
#[derive(Debug, Clone, Copy)]
pub struct Template3_0 {
    /// 地球の形状
    shape_of_earth: u8,
    /// 地球回転楕円体の長軸の尺度因子
    scale_factor_of_earth_major_axis: u8,
    /// 地球回転楕円体の長軸の尺度付きの長さ
    scaled_value_of_earth_major_axis: u32,
    /// 地球回転楕円体の短軸の尺度因子
    scale_factor_of_earth_minor_axis: u8,
    /// 地球回転楕円体の短軸の尺度付きの長さ
    scaled_value_of_earth_minor_axis: u32,
    /// 緯線に沿った格子点数
    number_of_along_lat_points: u32,
    /// 経線に沿った格子点数
    number_of_along_lon_points: u32,
    /// 原作成領域の基本角
    basic_angle_of_initial_product_domain: u32,
    /// 最初の格子点の緯度（10e-6度単位）
    lat_of_first_grid_point: u32,
    /// 最初の格子点の経度（10e-6度単位）
    lon_of_first_grid_point: u32,
    /// 分解能及び成分フラグ
    resolution_and_component_flags: u8,
    /// 最後の格子点の緯度（10e-6度単位）
    lat_of_last_grid_point: u32,
    /// 最後の格子点の経度（10e-6度単位）
    lon_of_last_grid_point: u32,
    /// i方向（経度方向）の増分（10e-6度単位）
    i_direction_increment: u32,
    /// j方向（緯度方向）の増分（10e-6度単位）
    j_direction_increment: u32,
    /// 走査モード
    scanning_mode: u8,
}

/// 第4節:プロダクト定義節
#[derive(Debug, Clone, Copy)]
pub struct Section4<T4> {
    /// 節の長さ
    section_bytes: usize,
    /// テンプレート直後の座標値の数
    number_of_after_template_points: u16,
    /// プロダクト定義テンプレート番号
    product_definition_template_number: u16,
    /// パラメータカテゴリー
    parameter_category: u8,
    /// テンプレート4
    template4: T4,
}

/// テンプレート4.50008
#[derive(Debug, Clone, Copy)]
pub struct Template4_50008 {
    /// パラメータ番号
    parameter_number: u8,
    /// 作成処理の種類
    type_of_generating_process: u8,
    /// 背景作成処理識別符
    background_process: u8,
    /// 観測資料の参照時刻からの締切時間（時）
    hours_after_data_cutoff: u16,
    /// 観測資料の参照時刻からの締切時間（分）
    minutes_after_data_cutoff: u8,
    /// 期間の単位の指示符
    indicator_of_unit_of_time_range: u8,
    /// 予報時間
    forecast_time: i32,
    /// 第一固定面の種類
    type_of_first_fixed_surface: u8,
    /// 全時間間隔の終了時
    end_of_all_time_intervals: PrimitiveDateTime,
    /// 統計を算出するために使用した時間間隔を記述する期間の仕様の数
    number_of_time_range_specs: u8,
    /// 統計処理における欠測資料の総数
    number_of_missing_values: u32,
    /// 統計処理の種類
    type_of_stat_proc: u8,
    /// 統計処理の時間増分の種類
    type_of_stat_proc_time_increment: u8,
    /// 統計処理の時間の単位の指示符
    stat_proc_time_unit: u8,
    /// 統計処理した時間の長さ
    stat_proc_time_length: u32,
    /// 連続的な資料場間の増分に関する時間の単位の指示符
    successive_time_unit: u8,
    /// 連続的な資料場間の時間の増分
    successive_time_increment: u32,
    /// レーダー等運用情報その1
    radar_info1: u64,
    /// レーダー等運用情報その2
    radar_info2: u64,
    /// 雨量計運用情報
    rain_gauge_info: u64,
}

/// 第5節:資料表現節
#[derive(Debug, Clone, Copy)]
pub struct Section5<T5> {
    /// 節の長さ
    section_bytes: usize,
    /// 全資料点の数
    number_of_values: u32,
    /// 資料表現テンプレート番号
    data_representation_template_number: u16,
    /// 1データのビット数
    bits_per_value: u8,
    /// テンプレート5
    template5: T5,
}

/// テンプレート5.200
#[derive(Debug, Clone)]
pub struct Template5_200 {
    /// 今回の圧縮に用いたレベルの最大値
    max_level_value: u16,
    /// データの取り得るレベルの最大値
    number_of_level_values: u16,
    /// データ代表値の尺度因子
    decimal_scale_factor: u8,
    /// レベルmに対応するデータ代表値
    /// レベル値と物理値(mm/h)の対応を格納するコレクション
    level_values: Vec<u16>,
}

/// 第6節:ビットマップ節
#[derive(Debug, Clone, Copy)]
pub struct Section6 {
    /// 節の長さ
    section_bytes: usize,
    /// ビットマップ指示符
    bitmap_indicator: u8,
}

/// 第７節:資料節
pub struct Section7<T7> {
    /// 節の長さ
    section_bytes: usize,
    /// テンプレート7
    template7: T7,
}

/// テンプレート7.200
pub struct Template7_200 {
    /// ランレングス圧縮符号列の開始位置
    run_length_position: usize,
    /// ランレングス圧縮符号列のバイト数
    run_length_bytes: usize,
}

/// 第８節:終端節
pub struct Section8;

/// テンプレート番号を検証する文を展開するマクロ
macro_rules! validate_template_number {
    ($name:literal, $template_number:ident, $expected:ident) => {
        if $template_number != $expected {
            return Err(ReaderError::ReadError(
                format!(
                    "{}が{}であることを想定していましたが{}でした。",
                    $name, $expected, $template_number
                )
                .into(),
            ));
        }
    };
}

impl FromReader for Section0 {
    /// GRIB2ファイルから第0節:指示節を読み込む。
    ///
    /// # 引数
    ///
    /// * `reader` - GRIB2ファイルリーダー
    ///
    /// # 戻り値
    ///
    /// 第0節: 指示節
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // GRIB: 4バイト
        validate_str(reader, "第0節:GRIB", 4, "GRIB")?;
        // 保留: 2バイト
        seek_relative(reader, "第0節:保留", 2)?;
        // 資料分野: 1バイト
        let discipline = read_u8(reader, "第0節:資料分野")?;
        // GRIB版番号: 1バイト
        let edition_number = validate_u8(reader, EDITION_NUMBER, "第0節:GRIB版番号")?;
        // GRIB報全体の長さ: 8バイト
        let total_length = read_u64(reader, "第0節:GRIB報全体の長さ")? as usize;

        Ok(Self {
            discipline,
            edition_number,
            total_length,
        })
    }
}

impl Section0 {
    /// 第0節:指示節を出力する。
    #[rustfmt::skip]
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第0節:指示節")?;
        writeln!(writer, "    資料分野: {}", self.discipline)?;
        writeln!(writer, "    GRIB版番号: {}", self.edition_number)?;
        writeln!(writer, "    GRIB報全体の長さ: 0x{:08X}", self.total_length)?;

        Ok(())
    }
}

impl FromReader for Section1 {
    /// GRIB2ファイルから第1節:識別節を読み込む。
    ///
    /// # 引数
    ///
    /// * `reader` - GRIB2ファイルリーダー
    ///
    /// # 戻り値
    ///
    /// 第1節: 識別節
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // 節の長さ: 4bytes
        let section_bytes = validate_u32(reader, SECTION1_BYTES, "第1節:節の長さ")? as usize;
        // 節番号
        validate_u8(reader, 1, "第1節:節番号")?;
        // 作成中枢の識別: 2bytes
        let center = read_u16(reader, "第1節:作成中枢")?;
        // 作成副中枢: 2bytes
        let sub_center = read_u16(reader, "第1節:作成副中枢")?;
        // GRIBマスター表バージョン番号: 1byte
        let table_version = read_u8(reader, "第1節:GRIBマスター表バージョン番号")?;
        // GRIB地域表バージョン番号: 1byte
        let local_table_version = read_u8(reader, "第1節:GRIB地域表バージョン番号")?;
        // 参照時刻の意味: 1byte
        let significance_of_reference_time = read_u8(reader, "第1節:参照時刻の意味")?;
        // 資料の参照時刻（日時）
        let referenced_at = read_datetime(reader, "第1節:資料の参照時刻")?;
        // 作成ステータス
        let production_status_of_processed_data = read_u8(reader, "第1節:作成ステータス")?;
        // 資料の種類
        let type_of_processed_data = read_u8(reader, "第1節:資料の種類")?;

        Ok(Self {
            section_bytes,
            center,
            sub_center,
            table_version,
            local_table_version,
            significance_of_reference_time,
            referenced_at,
            production_status_of_processed_data,
            type_of_processed_data,
        })
    }
}

impl Section1 {
    /// 第1節:識別節を出力する。
    #[rustfmt::skip]
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第1節:識別節")?;
        writeln!(writer, "    節の長さ: {}", self.section_bytes)?;
        writeln!(writer, "    作成中枢の識別: {}", self.center)?;
        writeln!(writer, "    作成副中枢: {}", self.sub_center)?;
        writeln!(writer, "    GRIBマスター表バージョン番号: {}", self.table_version)?;
        writeln!(writer, "    GRIB地域表バージョン番号: {}", self.local_table_version)?;
        writeln!(writer, "    参照時刻の意味: {}", self.significance_of_reference_time)?;
        writeln!(writer, "    資料の参照時刻: {}", self.referenced_at)?;
        writeln!(writer, "    作成ステータス: {}", self.production_status_of_processed_data)?;
        writeln!(writer, "    資料の種類: {}", self.type_of_processed_data)?;

        Ok(())
    }
}

impl Section2 {
    /// GRIB2ファイルから第2節:地域使用節を読み込む。
    ///
    /// # 引数
    ///
    /// * `_reader` - GRIB2ファイルリーダー
    ///
    /// # 戻り値
    ///
    /// 第2節:地域使用節
    pub(crate) fn from_reader(_reader: &mut FileReader) -> ReaderResult<Self> {
        Ok(Self)
    }

    /// 第2節:地域使用節を出力する。
        #[rustfmt::skip]
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
        where
            W: std::io::Write,
        {
            writeln!(writer, "第2節:地域使用節")?;

            Ok(())
        }
}

impl<T3> FromReader for Section3<T3>
where
    T3: TemplateFromReader<u16>,
{
    /// GRIB2ファイルから第3節:格子系定義節を読み込む。
    ///
    /// # 引数
    ///
    /// * `reader` - GRIB2ファイルリーダー
    ///
    /// # 戻り値
    ///
    /// 第3節: 格子系定義節
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // 節の長さ: 4バイト
        let section_bytes = read_u32(reader, "第3節:節の長さ")? as usize;
        // 節番号: 1バイト
        validate_u8(reader, 3, "第3節:節番号")?;
        // 格子系定義の出典: 1バイト
        let source_of_grid_definition = read_u8(reader, "第3節:格子系定義の出典")?;
        // 資料点数: 4バイト
        let number_of_data_points = read_u32(reader, "第3節:格子点数")?;
        // 格子点数を定義するリストのオクテット数: 1バイト
        let number_of_octets_for_number_of_points =
            read_u8(reader, "第3節:格子点数を定義するリストのオクテット数")?;
        // 格子点数を定義するリストの説明
        let interpretation_of_number_of_points =
            read_u8(reader, "第3節:格子点数を定義するリストの説明")?;
        // 格子系定義テンプレート番号: 2バイト
        let grid_definition_template_number = read_u16(reader, "第3節:格子系定義テンプレート番号")?;
        // テンプレート3
        let template = T3::from_reader(reader, grid_definition_template_number)?;

        Ok(Self {
            section_bytes,
            source_of_grid_definition,
            number_of_data_points,
            number_of_octets_for_number_of_points,
            interpretation_of_number_of_points,
            grid_definition_template_number,
            template3: template,
        })
    }
}

impl<T3> Section3<T3> {
    #[rustfmt::skip]
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
    T3: DebugTemplate<W>,
        W: std::io::Write,
    {
        writeln!(writer, "第3節:格子系定義節")?;
        writeln!(writer, "    節の長さ: {}", self.section_bytes)?;
        writeln!(writer, "    格子系定義の出典: {}", self.source_of_grid_definition())?;
        writeln!(writer, "    資料点数: {}", self.number_of_data_points)?;
        writeln!(writer, "    格子点数を定義するリストのオクテット数: {}", self.number_of_octets_for_number_of_points)?;
        writeln!(writer, "    格子点数を定義するリストの説明: {}", self.interpretation_of_number_of_points)?;
        writeln!(writer, "    格子系定義テンプレート番号: {}", self.grid_definition_template_number)?;
        self.template3.debug_info(writer)?;

        Ok(())
    }
}

impl Section3<Template3_0> {
    /// 地球の形状を返す。
    ///
    /// # 戻り値
    ///
    /// 地球の形状
    pub fn shape_of_earth(&self) -> u8 {
        self.template3.shape_of_earth
    }

    /// 地球回転楕円体の長軸の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の長軸の尺度因子
    pub fn scale_factor_of_earth_major_axis(&self) -> u8 {
        self.template3.scale_factor_of_earth_major_axis
    }

    /// 地球回転楕円体の長軸の尺度付きの長さを返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の長軸の尺度付きの長さ
    pub fn scaled_value_of_earth_major_axis(&self) -> u32 {
        self.template3.scaled_value_of_earth_major_axis
    }

    /// 地球回転楕円体の短軸の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の短軸の尺度因子
    pub fn scale_factor_of_earth_minor_axis(&self) -> u8 {
        self.template3.scale_factor_of_earth_minor_axis
    }

    /// 地球回転楕円体の短軸の尺度付きの長さを返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の短軸の尺度付きの長さ
    pub fn scaled_value_of_earth_minor_axis(&self) -> u32 {
        self.template3.scaled_value_of_earth_minor_axis
    }

    /// 緯線に沿った格子点数を返す。
    ///
    /// # 戻り値
    ///
    /// 緯線に沿った格子点数
    pub fn number_of_along_lat_points(&self) -> u32 {
        self.template3.number_of_along_lat_points
    }

    /// 経線に沿った格子点数を返す。
    ///
    /// # 戻り値
    ///
    /// 経線に沿った格子点数
    pub fn number_of_along_lon_points(&self) -> u32 {
        self.template3.number_of_along_lon_points
    }

    /// 原作成領域の基本角を返す。
    ///
    /// # 戻り値
    ///
    /// 原作成領域の基本角
    pub fn basic_angle_of_initial_product_domain(&self) -> u32 {
        self.template3.basic_angle_of_initial_product_domain
    }

    /// 最初の格子点の緯度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最初の格子点の緯度（10e-6度単位）
    pub fn lat_of_first_grid_point(&self) -> u32 {
        self.template3.lat_of_first_grid_point
    }

    /// 最初の格子点の経度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最初の格子点の経度（10e-6度単位）
    pub fn lon_of_first_grid_point(&self) -> u32 {
        self.template3.lon_of_first_grid_point
    }

    /// 分解能及び成分フラグを返す。
    ///
    /// # 戻り値
    ///
    /// 分解能及び成分フラグ
    pub fn resolution_and_component_flags(&self) -> u8 {
        self.template3.resolution_and_component_flags
    }

    /// 最後の格子点の緯度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最後の格子点の緯度（10e-6度単位）
    pub fn lat_of_last_grid_point(&self) -> u32 {
        self.template3.lat_of_last_grid_point
    }

    /// 最後の格子点の経度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最後の格子点の経度（10e-6度単位）
    pub fn lon_of_last_grid_point(&self) -> u32 {
        self.template3.lon_of_last_grid_point
    }

    /// i方向（経度方向）の増分（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// i方向（経度方向）の増分（10e-6度単位）
    pub fn i_direction_increment(&self) -> u32 {
        self.template3.i_direction_increment
    }

    /// j方向（緯度方向）の増分（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// j方向（緯度方向）の増分（10e-6度単位）
    pub fn j_direction_increment(&self) -> u32 {
        self.template3.j_direction_increment
    }

    /// 走査モードを返す。
    ///
    /// # 戻り値
    ///
    /// 走査モード
    pub fn scanning_mode(&self) -> u8 {
        self.template3.scanning_mode
    }
}

impl TemplateFromReader<u16> for Template3_0 {
    fn from_reader(reader: &mut FileReader, template_number: u16) -> ReaderResult<Self> {
        // 格子系定義テンプレート番号を確認
        validate_template_number!(
            "第3節:格子系定義テンプレート番号",
            template_number,
            LAT_LON_GRID_DEFINITION_TEMPLATE_NUMBER
        );
        // 地球の形状: 1バイト
        let shape_of_earth = read_u8(reader, "第3節:地球の形状")?;
        // 地球球体の半径の尺度因子: 1バイト
        seek_relative(reader, "第3節:地球球体の半径の尺度因子", 1)?;
        // 地球球体の尺度付き半径: 4バイト
        seek_relative(reader, "第3節:地球球体の尺度付き半径", 4)?;
        // 地球回転楕円体の長軸の尺度因子: 1バイト
        let scale_factor_of_earth_major_axis =
            read_u8(reader, "第3節:地球回転楕円体の長軸の尺度因子")?;
        // 地球回転楕円体の長軸の尺度付きの長さ: 4バイト
        let scaled_value_of_earth_major_axis =
            read_u32(reader, "第3節:地球回転楕円体の長軸の尺度付きの長さ")?;
        // 地球回転楕円体の短軸の尺度因子: 1バイト
        let scale_factor_of_earth_minor_axis =
            read_u8(reader, "第3節:地球回転楕円体の短軸の尺度因子")?;
        // 地球回転楕円体の短軸の尺度付きの長さ: 4バイト
        let scaled_value_of_earth_minor_axis =
            read_u32(reader, "第3節:地球回転楕円体の短軸の尺度付きの長さ")?;
        // 緯線に沿った格子点数: 4バイト
        let number_of_along_lat_points = read_u32(reader, "第3節:緯線に沿った格子点数")?;
        // 経線に沿った格子点数: 4バイト
        let number_of_along_lon_points = read_u32(reader, "第3節:経線に沿った格子点数")?;
        // 原作成領域の基本角: 4バイト
        let basic_angle_of_initial_product_domain = read_u32(reader, "第3節:原作成領域の基本角")?;
        // 端点の経度及び緯度並びに方向増分の定義に使われる基本角の細分: 4バイト
        seek_relative(reader, "第3節:端点の経度及び緯度並びに方向増分の定義", 4)?;
        // 最初の格子点の緯度（10e-6度単位）: 4バイト
        let lat_of_first_grid_point = read_u32(reader, "第3節:最初の格子点の緯度")?;
        // 最初の格子点の経度（10e-6度単位）: 4バイト
        let lon_of_first_grid_point = read_u32(reader, "第3節:最初の格子点の経度")?;
        // 分解能及び成分フラグ: 1バイト
        let resolution_and_component_flags = read_u8(reader, "第3節:分解能及び成分フラグ")?;
        // 最後の格子点の緯度（10e-6度単位）: 4バイト
        let lat_of_last_grid_point = read_u32(reader, "第3節:最後の格子点の緯度")?;
        // 最後の格子点の経度（10e-6度単位）: 4バイト
        let lon_of_last_grid_point = read_u32(reader, "第3節:最後の格子点の経度")?;
        // i方向（経度方向）の増分（10e-6度単位）: 4バイト
        let i_direction_increment = read_u32(reader, "第3節:i方向の増分")?;
        // j方向（緯度方向）の増分（10e-6度単位）: 4バイト
        let j_direction_increment = read_u32(reader, "第3節:j方向の増分")?;
        // 走査モード: 1バイト
        let scanning_mode = read_u8(reader, "第3節:走査モード")?;

        Ok(Self {
            shape_of_earth,
            scale_factor_of_earth_major_axis,
            scaled_value_of_earth_major_axis,
            scale_factor_of_earth_minor_axis,
            scaled_value_of_earth_minor_axis,
            number_of_along_lat_points,
            number_of_along_lon_points,
            basic_angle_of_initial_product_domain,
            lat_of_first_grid_point,
            lon_of_first_grid_point,
            resolution_and_component_flags,
            lat_of_last_grid_point,
            lon_of_last_grid_point,
            i_direction_increment,
            j_direction_increment,
            scanning_mode,
        })
    }
}

impl<W> DebugTemplate<W> for Template3_0 {
    #[rustfmt::skip]
    fn debug_info(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "    地球の形状: {}", self.shape_of_earth)?;
        writeln!(writer, "    地球回転楕円体の長軸の尺度因子: {}", self.scale_factor_of_earth_major_axis)?;
        writeln!(writer, "    地球回転楕円体の長軸の尺度付きの長さ: {}", self.scaled_value_of_earth_major_axis)?;
        writeln!(writer, "    地球回転楕円体の短軸の尺度因子: {}", self.scale_factor_of_earth_minor_axis)?;
        writeln!(writer, "    地球回転楕円体の短軸の尺度付きの長さ: {}", self.scaled_value_of_earth_minor_axis)?;
        writeln!(writer, "    緯線に沿った格子点数: {}", self.number_of_along_lat_points)?;
        writeln!(writer, "    経線に沿った格子点数: {}", self.number_of_along_lon_points)?;
        writeln!(writer, "    原作成領域の基本角: {}", self.basic_angle_of_initial_product_domain)?;
        writeln!(writer, "    最初の格子点の緯度: {}", self.lat_of_first_grid_point)?;
        writeln!(writer, "    最初の格子点の経度: {}", self.lon_of_first_grid_point)?;
        writeln!(writer, "    分解能及び成分フラグ: 0x{:02X}", self.resolution_and_component_flags)?;
        writeln!(writer, "    最後の格子点の緯度: {}", self.lat_of_last_grid_point)?;
        writeln!(writer, "    最後の格子点の経度: {}", self.lon_of_last_grid_point)?;
        writeln!(writer, "    i方向の増分: {}", self.j_direction_increment)?;
        writeln!(writer, "    j方向の増分: {}", self.i_direction_increment)?;
        writeln!(writer, "    走査モード: 0x{:02X}", self.scanning_mode)?;

        Ok(())
    }
}

impl<T4> FromReader for Section4<T4>
where
    T4: TemplateFromReader<u16>,
{
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // 節の長さ: 4バイト
        let section_bytes = read_u32(reader, "第4節:節の長さ")? as usize;
        // 節番号: 1バイト
        validate_u8(reader, 4, "第4節:節番号")?;
        // テンプレート直後の座標値の数: 2バイト
        let number_of_after_template_points =
            read_u16(reader, "第4節:テンプレート直後の座標値の数")?;
        // プロダクト定義テンプレート番号: 2バイト
        let product_definition_template_number =
            read_u16(reader, "第4節:プロダクト定義テンプレート番号")?;
        // パラメータカテゴリー: 1バイト
        let parameter_category = read_u8(reader, "第4節:パラメータカテゴリー")?;
        // テンプレート4
        let template4 = T4::from_reader(reader, product_definition_template_number)?;

        Ok(Self {
            section_bytes,
            number_of_after_template_points,
            product_definition_template_number,
            parameter_category,
            template4,
        })
    }
}

impl<T4> Section4<T4> {
    /// 節の長さを返す。
    ///
    /// # 戻り値
    ///
    /// 節の長さ
    pub fn section_bytes(&self) -> usize {
        self.section_bytes
    }

    /// テンプレート直後の座標値の数を返す。
    ///
    /// # 戻り値
    ///
    /// テンプレート直後の座標値の数
    pub fn number_of_after_template_points(&self) -> u16 {
        self.number_of_after_template_points
    }

    /// プロダクト定義テンプレート番号を返す。
    ///
    /// # 戻り値
    ///
    /// プロダクト定義テンプレート番号
    pub fn product_definition_template_number(&self) -> u16 {
        self.product_definition_template_number
    }

    /// パラメータカテゴリーを返す。
    ///
    /// # 戻り値
    ///
    /// パラメータカテゴリー
    pub fn parameter_category(&self) -> u8 {
        self.parameter_category
    }

    /// 第4節:プロダクト定義節を出力する。
    #[rustfmt::skip]
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
        T4: DebugTemplate<W>,
    {
        writeln!(writer, "第4節:プロダクト定義節")?;
        writeln!(writer, "    節の長さ: {}", self.section_bytes)?;
        writeln!(writer, "    テンプレート直後の座標値の数: {}", self.number_of_after_template_points)?;
        writeln!(writer, "    プロダクト定義テンプレート番号: {}", self.product_definition_template_number)?;
        writeln!(writer, "    パラメータカテゴリー: {}", self.parameter_category)?;
        self.template4.debug_info(writer)?;

        Ok(())
    }
}

impl Section4<Template4_50008> {
    /// パラメータ番号を返す。
    ///
    /// # 戻り値
    ///
    /// パラメータ番号
    pub fn parameter_number(&self) -> u8 {
        self.template4.parameter_number
    }

    /// 作成処理の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 作成処理の種類
    pub fn type_of_generating_process(&self) -> u8 {
        self.template4.type_of_generating_process
    }

    /// 背景作成処理識別符を返す。
    ///
    /// # 戻り値
    ///
    /// 背景作成処理識別符
    pub fn background_process(&self) -> u8 {
        self.template4.background_process
    }

    /// 観測資料の参照時刻からの締切時間（時）を返す。
    ///
    /// # 戻り値
    ///
    /// 観測資料の参照時刻からの締切時間（時）
    pub fn hours_after_data_cutoff(&self) -> u16 {
        self.template4.hours_after_data_cutoff
    }

    /// 観測資料の参照時刻からの締切時間（分）を返す。
    ///
    /// # 戻り値
    ///
    /// 観測資料の参照時刻からの締切時間（分）
    pub fn minutes_after_data_cutoff(&self) -> u8 {
        self.template4.minutes_after_data_cutoff
    }

    /// 期間の単位の指示符を返す。
    ///
    /// # 戻り値
    ///
    /// 期間の単位の指示符
    pub fn indicator_of_unit_of_time_range(&self) -> u8 {
        self.template4.indicator_of_unit_of_time_range
    }

    /// 予報時間を返す。
    ///
    /// # 戻り値
    ///
    /// 予報時間
    pub fn forecast_time(&self) -> i32 {
        self.template4.forecast_time
    }

    /// 第一固定面の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 第一固定面の種類
    pub fn first_fixed_surface_type(&self) -> u8 {
        self.template4.type_of_first_fixed_surface
    }

    /// 全時間間隔の終了時を返す。
    ///
    /// # 戻り値
    ///
    /// 全時間間隔の終了時
    pub fn end_of_all_time_intervals(&self) -> PrimitiveDateTime {
        self.template4.end_of_all_time_intervals
    }

    /// 統計を算出するために使用した時間間隔を記述する期間の仕様の数を返す。
    ///
    /// # 戻り値
    ///
    /// 統計を算出するために使用した時間間隔を記述する期間の仕様の数
    pub fn number_of_time_range_specs(&self) -> u8 {
        self.template4.number_of_time_range_specs
    }

    /// 統計処理における欠測資料の総数を返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理における欠測資料の総数
    pub fn number_of_missing_values(&self) -> u32 {
        self.template4.number_of_missing_values
    }

    /// 統計処理の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理の種類
    pub fn type_of_stat_proc(&self) -> u8 {
        self.template4.type_of_stat_proc
    }

    /// 統計処理の時間増分の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理の時間増分の種類
    pub fn type_of_stat_proc_time_increment(&self) -> u8 {
        self.template4.type_of_stat_proc_time_increment
    }

    /// 統計処理の時間の単位の指示符を返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理の時間の単位の指示符
    pub fn stat_proc_time_unit(&self) -> u8 {
        self.template4.stat_proc_time_unit
    }

    /// 統計処理した時間の長さを返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理した時間の長さ
    pub fn stat_proc_time_length(&self) -> u32 {
        self.template4.stat_proc_time_length
    }

    /// 連続的な資料場間の増分に関する時間の単位の指示符を返す。
    ///
    /// # 戻り値
    ///
    /// 連続的な資料場間の増分に関する時間の単位の指示符
    pub fn successive_time_unit(&self) -> u8 {
        self.template4.successive_time_unit
    }

    /// 連続的な資料場間の時間の増分を返す。
    ///
    /// # 戻り値
    ///
    /// 連続的な資料場間の時間の増分
    pub fn successive_time_increment(&self) -> u32 {
        self.template4.successive_time_increment
    }

    /// レーダー等運用情報その1を返す。
    ///
    /// # 戻り値
    ///
    /// レーダー等運用情報その1
    pub fn radar_info1(&self) -> u64 {
        self.template4.radar_info1
    }

    /// レーダー等運用情報その2を返す。
    ///
    /// # 戻り値
    ///
    /// レーダー等運用情報その2
    pub fn radar_info2(&self) -> u64 {
        self.template4.radar_info2
    }

    /// 雨量計運用情報を返す。
    ///
    /// # 戻り値
    ///
    /// 雨量計運用情報
    pub fn rain_gauge_info(&self) -> u64 {
        self.template4.rain_gauge_info
    }
}

impl TemplateFromReader<u16> for Template4_50008 {
    fn from_reader(reader: &mut FileReader, template_number: u16) -> ReaderResult<Self> {
        // プロダクト定義テンプレート番号を確認
        validate_template_number!(
            "第4節:プロダクト定義テンプレート番号",
            template_number,
            RADAR_PRODUCT_DEFINITION_TEMPLATE_NUMBER
        );
        // パラメータ番号: 1バイト
        let parameter_number = read_u8(reader, "第4節:パラメータ番号")?;
        // 作成処理の種類: 1バイト
        let type_of_generating_process = read_u8(reader, "第4節:作成処理の種類")?;
        // 背景作成処理識別符: 1バイト
        let background_process = read_u8(reader, "第4節:背景作成処理識別符")?;
        // 予報の作成処理識別符: 1バイト
        seek_relative(reader, "第4節:予報の作成処理識別符", 1)?;
        // 観測資料の参照時刻からの締切時間（時）: 2バイト
        let hours_after_data_cutoff =
            read_u16(reader, "第4節:観測資料の参照時刻からの締切時間（時）")?;
        // 観測資料の参照時刻からの締切時間（分）: 1バイト
        let minutes_after_data_cutoff =
            read_u8(reader, "第4節:観測資料の参照時刻からの締切時間（分）")?;
        // 期間の単位の指示符: 1バイト
        let indicator_of_unit_of_time_range = read_u8(reader, "第4節:期間の単位の指示符")?;
        // 予報時間: 4バイト
        let forecast_time = read_i32(reader, "第4節:予報時間")?;
        // 第一固定面の種類: 1バイト
        let type_of_first_fixed_surface = read_u8(reader, "第4節:第一固定面の種類")?;
        // 第一固定面の尺度因子: 1バイト
        seek_relative(reader, "第4節:第一固定面の尺度因子", 1)?;
        // 第一固定面の尺度付きの値: 4バイト
        seek_relative(reader, "第4節:第一固定面の尺度付きの値", 4)?;
        // 第二固定面の種類: 1バイト
        seek_relative(reader, "第4節:第二固定面の種類", 1)?;
        // 第二固定面の尺度因子: 1バイト
        seek_relative(reader, "第4節:第二固定面の尺度因子", 1)?;
        // 第二固定面の尺度付きの値: 4バイト
        seek_relative(reader, "第4節:第二固定面の尺度付きの値", 4)?;
        // 全時間間隔の終了時: 7バイト
        let end_of_all_time_intervals = read_datetime(reader, "第4節:全時間間隔の終了時")?;
        // 統計を算出するために使用した時間間隔を記述する期間の仕様の数: 1バイト
        let number_of_time_range_specs = read_u8(
            reader,
            "第4節:統計を算出するために使用した時間間隔を記述する期間の仕様の数",
        )?;
        // 統計処理における欠測資料の総数: 4バイト
        let number_of_missing_values = read_u32(reader, "第4節:統計処理における欠測資料の総数")?;
        // 統計処理の種類: 1バイト
        let type_of_stat_proc = read_u8(reader, "第4節:統計処理の種類")?;
        // 統計処理の時間増分の種類: 1バイト
        let type_of_stat_proc_time_increment = read_u8(reader, "第4節:統計処理の時間増分の種類")?;
        // 統計処理の時間の単位の指示符: 1バイト
        let stat_proc_time_unit = read_u8(reader, "第4節:統計処理の時間の単位の指示符")?;
        // 統計処理した期間の長さ: 4バイト
        let stat_proc_time_length = read_u32(reader, "第4節:統計処理の時間増分の長さ")?;
        // 連続的な資料場間の増分に関する時間の単位の指示符: 1バイト
        let successive_time_unit = read_u8(
            reader,
            "第4節:連続的な資料場間の増分に関する時間の単位の指示符",
        )?;
        // 連続的な資料場間の時間の増分: 4バイト
        let successive_time_increment = read_u32(reader, "第4節:連続的な資料場間の時間の増分")?;
        // レーダー等運用情報その1: 8バイト
        let radar_info1 = read_u64(reader, "第4節:レーダー等運用情報その1")?;
        // レーダー等運用情報その2: 8バイト
        let radar_info2 = read_u64(reader, "第4節:レーダー等運用情報その2")?;
        // 雨量計運用情報: 8バイト
        let rain_gauge_info = read_u64(reader, "第4節:雨量計運用情報の読み込みに失敗しました。")?;

        Ok(Self {
            parameter_number,
            type_of_generating_process,
            background_process,
            hours_after_data_cutoff,
            minutes_after_data_cutoff,
            indicator_of_unit_of_time_range,
            forecast_time,
            type_of_first_fixed_surface,
            end_of_all_time_intervals,
            number_of_time_range_specs,
            number_of_missing_values,
            type_of_stat_proc,
            type_of_stat_proc_time_increment,
            stat_proc_time_unit,
            stat_proc_time_length,
            successive_time_unit,
            successive_time_increment,
            radar_info1,
            radar_info2,
            rain_gauge_info,
        })
    }
}

impl<W> DebugTemplate<W> for Template4_50008 {
    #[rustfmt::skip]
    fn debug_info(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "    パラメータ番号: {}", self.parameter_number)?;
        writeln!(writer, "    作成処理の種類: {}", self.type_of_generating_process)?;
        writeln!(writer, "    背景作成処理識別符: {}", self.background_process)?;
        writeln!(writer, "    観測資料の参照時刻からの締切時間(時): {}", self.hours_after_data_cutoff)?;
        writeln!(writer, "    観測資料の参照時刻からの締切時間(分): {}", self.minutes_after_data_cutoff)?;
        writeln!(writer, "    期間の単位の指示符: {}", self.indicator_of_unit_of_time_range)?;
        writeln!(writer, "    予報時間（分）: {}", self.forecast_time)?;
        writeln!(writer, "    第一固定面の種類: {}", self.type_of_first_fixed_surface)?;
        writeln!(writer, "    全時間間隔の終了時: {}", self.end_of_all_time_intervals)?;
        writeln!(writer, "    統計を算出するために使用した時間間隔を記述する期間の仕様の数: {}", self.number_of_time_range_specs)?;
        writeln!(writer, "    統計処理における欠測資料の総数: {}", self.number_of_missing_values)?;
        writeln!(writer, "    統計処理の種類: {}", self.type_of_stat_proc)?;
        writeln!(writer, "    統計処理の時間増分の種類: {}", self.type_of_stat_proc_time_increment)?;
        writeln!(writer, "    統計処理の時間の単位の指示符: {}", self.stat_proc_time_unit)?;
        writeln!(writer, "    統計処理した期間の長さ: {}", self.stat_proc_time_length)?;
        writeln!(writer, "    連続的な資料場間の増分に関する時間の単位の指示符: {}", self.successive_time_unit)?;
        writeln!(writer, "    続的な資料場間の時間の増分: {}", self.successive_time_increment)?;
        writeln!(writer, "    レーダー等運用情報その1: 0x{:02X}", self.radar_info1)?;
        writeln!(writer, "    レーダー等運用情報その2: 0x{:02X}", self.radar_info2)?;
        writeln!(writer, "    雨量計運用情報: 0x{:02X}", self.rain_gauge_info)?;

        Ok(())
    }
}

impl<T5> FromReader for Section5<T5>
where
    T5: TemplateFromReaderWithSize<u16>,
{
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // 節の長さ: 4バイト
        let section_bytes = read_u32(reader, "第5節:節の長さ")? as usize;
        // 節番号: 1バイト
        validate_u8(reader, 5, "第5節:節番号")?;
        // 全資料点の数: 4バイト
        let number_of_values = read_u32(reader, "第5節:全資料点の数")?;
        // 資料表現テンプレート番号: 2バイト
        let data_representation_template_number =
            read_u16(reader, "第5節:資料表現テンプレート番号")?;
        // 1データのビット数: 1バイト
        let bits_per_value = read_u8(reader, "第5節:1データのビット数")?;
        // テンプレート5
        let template_bytes = section_bytes - (4 + 1 + 4 + 2 + 1);
        let template5 =
            T5::from_reader(reader, data_representation_template_number, template_bytes)?;

        Ok(Self {
            section_bytes,
            number_of_values,
            data_representation_template_number,
            bits_per_value,
            template5,
        })
    }
}

impl<T5> Section5<T5> {
    /// 節の長さを返す。
    ///
    /// # 戻り値
    ///
    /// 節の長さ
    pub fn section_bytes(&self) -> usize {
        self.section_bytes
    }

    /// 全資料点の数を返す。
    ///
    /// # 戻り値
    ///
    /// 全資料点の数
    pub fn number_of_values(&self) -> u32 {
        self.number_of_values
    }

    /// 資料表現テンプレート番号を返す。
    ///
    /// # 戻り値
    ///
    /// 資料表現テンプレート番号
    pub fn data_representation_template_number(&self) -> u16 {
        self.data_representation_template_number
    }

    /// 1データのビット数を返す。
    ///
    /// # 戻り値
    ///
    /// 1データのビット数
    pub fn bits_per_value(&self) -> u8 {
        self.bits_per_value
    }

    /// 第5節:資料表現節を出力する。
    #[rustfmt::skip]
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
        T5: DebugTemplate<W>,
    {
        writeln!(writer, "第5節:資料表現節")?;
        writeln!(writer, "    節の長さ: {}", self.section_bytes())?;
        writeln!(writer, "    全資料点の数: {}", self.number_of_values())?;
        writeln!(writer, "    資料表現テンプレート番号: {}", self.data_representation_template_number())?;
        writeln!(writer, "    1データのビット数: {}", self.bits_per_value())?;
        self.template5.debug_info(writer)?;

        Ok(())
    }
}

impl Section5<Template5_200> {
    /// 今回の圧縮に用いたレベルの最大値を返す。
    ///
    /// # 戻り値
    ///
    /// 圧縮に用いたレベルの最大値
    pub fn max_level_value(&self) -> u16 {
        self.template5.max_level_value
    }

    /// データの取り得るレベルの最大値を返す。
    ///
    /// # 戻り値
    ///
    /// データの取り得るレベルの最大値
    pub fn number_of_level_values(&self) -> u16 {
        self.template5.number_of_level_values
    }

    /// データ代表値の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// データ代表値の尺度因子
    pub fn decimal_scale_factor(&self) -> u8 {
        self.template5.decimal_scale_factor
    }

    /// レベルmに対応するデータ代表値を返す。
    ///
    /// # 戻り値
    ///
    /// レベル値と物理値(mm/h)の対応を格納するコレクション
    pub fn level_values(&self) -> &[u16] {
        self.template5.level_values.as_ref()
    }
}

impl TemplateFromReaderWithSize<u16> for Template5_200 {
    fn from_reader(
        reader: &mut FileReader,
        template_number: u16,
        template_bytes: usize,
    ) -> ReaderResult<Self> {
        // 資料表現テンプレート番号を確認
        validate_template_number!(
            "第5節:資料表現テンプレート番号",
            template_number,
            RUN_LENGTH_DATA_REPRESENTATION_TEMPLATE_NUMBER
        );
        // 今回の圧縮に用いたレベルの最大値: 2バイト
        let max_level_value = read_u16(reader, "第5節:今回の圧縮に用いたレベルの最大値")?;
        // データの取り得るレベルの最大値: 2バイト
        let number_of_level_values = read_u16(reader, "第5節:レベルの最大値")?;
        // データ代表値の尺度因子: 1バイト
        let decimal_scale_factor = read_u8(reader, "第5節:データ代表値の尺度因子")?;
        // レベルmに対応するデータ代表値
        let number_of_levels = (template_bytes - (2 + 2 + 1)) / 2;
        let mut level_values = Vec::new();
        for _ in 0..number_of_levels {
            level_values.push(read_u16(reader, "第5節:レベルmに対応するデータ代表値")?);
        }

        Ok(Self {
            max_level_value,
            number_of_level_values,
            decimal_scale_factor,
            level_values,
        })
    }
}

impl<W> DebugTemplate<W> for Template5_200 {
    #[rustfmt::skip]
    fn debug_info(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "    今回の圧縮に用いたレベルの最大値: {}", self.max_level_value)?;
        writeln!(writer, "    データの取り得るレベルの最大値: {}", self.number_of_level_values)?;
        writeln!(writer, "    データ代表値の尺度因子: {}", self.decimal_scale_factor)?;
        writeln!(writer, "    レベルmに対応するデータ代表値:")?;
        for (i, level_value) in self.level_values.iter().enumerate() {
            writeln!(writer, "        レベル{}: {}", i + 1, level_value)?;
        }

        Ok(())
    }
}
impl FromReader for Section6 {
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // 節の長さ: 4バイト
        let section_bytes = validate_u32(reader, SECTION6_BYTES, "第6節:節の長さ")? as usize;
        // 節番号: 1バイト
        validate_u8(reader, 6, "第6節:節番号")?;
        // ビットマップ指示符: 1バイト
        let bitmap_indicator = read_u8(reader, "第6節:ビットマップ指示符")?;

        Ok(Self {
            section_bytes,
            bitmap_indicator,
        })
    }
}

impl Section6 {
    /// 節の長さを返す。
    ///
    /// # 戻り値
    ///
    /// 節の長さ
    pub fn section_bytes(&self) -> usize {
        self.section_bytes
    }

    /// ビットマップ指示符を返す。
    ///
    /// # 戻り値
    ///
    /// ビットマップ指示符
    pub fn bitmap_indicator(&self) -> u8 {
        self.bitmap_indicator
    }

    /// 第6節:ビットマップ節を出力する。
    #[rustfmt::skip]
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第6節:ビットマップ節")?;
        writeln!(writer, "    節の長さ: {}", self.section_bytes())?;
        writeln!(writer, "    ビットマップ指示符数: {}", self.bitmap_indicator())?;

        Ok(())
    }
}

impl<T7> FromReader for Section7<T7>
where
    T7: TemplateFromReaderWithSize<u16>,
{
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // 節の長さ: 4バイト
        let section_bytes = read_u32(reader, "第7節:節の長さ")? as usize;
        // 節番号: 1バイト
        validate_u8(reader, 7, "第7節:節番号")?;
        // テンプレート7
        let template_bytes = section_bytes - (4 + 1);
        let template7 = T7::from_reader(
            reader,
            RUN_LENGTH_DATA_REPRESENTATION_TEMPLATE_NUMBER,
            template_bytes,
        )?;

        Ok(Self {
            section_bytes,
            template7,
        })
    }
}

impl<T7> Section7<T7> {
    /// 節の長さを返す。
    ///
    /// # 戻り値
    ///
    /// 節の長さ
    pub fn section_bytes(&self) -> usize {
        self.section_bytes
    }

    /// 第7節:資料節を出力する。
    #[rustfmt::skip]
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
        T7: DebugTemplate<W>
    {
        writeln!(writer, "第7節:資料節")?;
        writeln!(writer, "    節の長さ: {}", self.section_bytes())?;
        self.template7.debug_info(writer)?;

        Ok(())
    }
}

impl Section7<Template7_200> {
    /// ランレングス圧縮符号列の開始位置を返す。
    ///
    /// # 戻り値
    ///
    /// ランレングス圧縮符号列の開始位置
    pub fn run_length_position(&self) -> usize {
        self.template7.run_length_position
    }

    /// ランレングス圧縮符号列のバイト数を返す。
    ///
    /// # 戻り値
    ///
    /// ランレングス圧縮符号列のバイト数
    pub fn run_length_bytes(&self) -> usize {
        self.template7.run_length_bytes
    }
}

impl TemplateFromReaderWithSize<u16> for Template7_200 {
    fn from_reader(
        reader: &mut FileReader,
        template_number: u16,
        template_bytes: usize,
    ) -> ReaderResult<Self> {
        // 資料表現テンプレート番号を確認
        validate_template_number!(
            "第7節:資料表現テンプレート番号",
            template_number,
            RUN_LENGTH_DATA_REPRESENTATION_TEMPLATE_NUMBER
        );
        // ランレングス圧縮符号列の開始位置を記憶
        let run_length_position = reader.stream_position().map_err(|_| {
            ReaderError::ReadError(
                "第7節:ランレングス圧縮符号列の開始位置の記憶に失敗しました。".into(),
            )
        })? as usize;

        // ランレングス圧縮符号列をスキップ
        reader.seek_relative(template_bytes as i64).map_err(|_| {
            ReaderError::ReadError(
                "第7節:ランレングス圧縮オクテット列の読み飛ばしに失敗しました。".into(),
            )
        })?;

        Ok(Self {
            run_length_position,
            run_length_bytes: template_bytes,
        })
    }
}

impl<W> DebugTemplate<W> for Template7_200 {
    #[rustfmt::skip]
    fn debug_info(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "    ランレングス圧縮符号開始位置: 0x{:08X}", self.run_length_position)?;
        writeln!(writer, "    ランレングス圧縮符号長さ: 0x{:08X}", self.run_length_bytes)?;

        Ok(())
    }
}

impl FromReader for Section8 {
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // 第8節:終端マーカー
        let value = read_str(reader, 4);
        match value {
            Ok(value) => {
                if value == SECTION8_END_MARKER {
                    Ok(Self {})
                } else {
                    Err(ReaderError::Unexpected(
                        format!(
                            "第8節の終了が不正です。ファイルを正確に読み込めなかった可能性があります。expected: {}, actual: {}",
                            SECTION8_END_MARKER, value
                        )
                        .into(),
                    ))
                }
            }
            Err(_) => Err(ReaderError::ReadError(
                "第8節の終了が不正です。ファイルを正確に読み込めなかった可能性があります。".into(),
            )),
        }
    }
}

fn seek_relative(reader: &mut FileReader, name: &str, offset: i64) -> ReaderResult<()> {
    reader
        .seek_relative(offset)
        .map_err(|_| ReaderError::ReadError(format!("{}を読み飛ばせませんでした。", name).into()))
}

fn validate_str(
    reader: &mut FileReader,
    name: &str,
    size: usize,
    expected: &str,
) -> ReaderResult<String> {
    let mut buf = vec![0; size];
    reader.read_exact(&mut buf).map_err(|_| {
        ReaderError::ReadError(format!("{}バイトの文字列の読み込みに失敗しました。", size).into())
    })?;
    let value = String::from_utf8(buf).map_err(|_| {
        ReaderError::Unexpected(format!("{}バイトの文字列のコードに失敗しました。", size).into())
    })?;
    if value != expected {
        return Err(ReaderError::Unexpected(
            format!(
                "{}の値は{}でしたが、{}でなければなりません。",
                name, value, expected
            )
            .into(),
        ));
    }

    Ok(value)
}

/// 数値を読み込む関数を生成するマクロ
macro_rules! read_number {
    ($fname:ident, $type:ty) => {
        fn $fname(reader: &mut FileReader, name: &str) -> ReaderResult<$type> {
            let expected_bytes = std::mem::size_of::<$type>();
            let mut buf = vec![0_u8; expected_bytes];
            reader.read_exact(&mut buf).map_err(|_| {
                ReaderError::ReadError(format!("{}の読み込みに失敗しました。", name).into())
            })?;

            Ok(<$type>::from_be_bytes(buf.try_into().unwrap()))
        }
    };
}

read_number!(read_u8, u8);
read_number!(read_u16, u16);
read_number!(read_u32, u32);
read_number!(read_u64, u64);
read_number!(read_i32, i32);

/// 数値を読み込み検証する関数を生成するマクロ
macro_rules! validate_number {
    ($fname:ident, $read_fn:ident, $type:ty) => {
        fn $fname(reader: &mut FileReader, expected: $type, name: &str) -> ReaderResult<$type> {
            let value = $read_fn(reader, name).map_err(|_| {
                ReaderError::ReadError(format!("{}の読み込みに失敗しました。", name).into())
            })?;
            if value != expected {
                return Err(ReaderError::Unexpected(
                    format!(
                        "{}の値は{}でしたが、{}でなければなりません。",
                        name, value, expected
                    )
                    .into(),
                ));
            }

            Ok(value)
        }
    };
}

validate_number!(validate_u8, read_u8, u8);
validate_number!(validate_u32, read_u32, u32);

fn read_str(reader: &mut FileReader, size: usize) -> ReaderResult<String> {
    let mut buf = vec![0; size];
    reader.read_exact(&mut buf).map_err(|_| {
        ReaderError::ReadError(format!("{}バイトの文字列の読み込みに失敗しました。", size).into())
    })?;

    Ok(String::from_utf8(buf).map_err(|_| {
        ReaderError::Unexpected(format!("{}バイトの文字列のコードに失敗しました。", size).into())
    }))?
}

fn read_datetime(reader: &mut FileReader, name: &str) -> ReaderResult<PrimitiveDateTime> {
    let year = read_u16(reader, name)?;
    let mut parts = Vec::new();
    for _ in 0..5 {
        parts.push(read_u8(reader, name)?);
    }
    // 日付と時刻を構築
    let month = Month::try_from(parts[0]).map_err(|_| {
        ReaderError::Unexpected(
            format!(
                "{}:月の値は{}でしたが、1から12の範囲でなければなりません。",
                name, parts[0]
            )
            .into(),
        )
    })?;
    let date = Date::from_calendar_date(year as i32, month, parts[1]).map_err(|_| {
        ReaderError::Unexpected(
            format!(
                "{}:{}年{}月は{}日を日付に変換できませんでした。",
                name, year, month as u8, parts[1]
            )
            .into(),
        )
    })?;
    let time = Time::from_hms(parts[2], parts[3], parts[4]).map_err(|_| {
        ReaderError::Unexpected(
            format!(
                "{}:{}時{}分{}秒を時刻に変換できませんでした。",
                name, parts[2], parts[3], parts[4]
            )
            .into(),
        )
    })?;

    Ok(PrimitiveDateTime::new(date, time))
}

pub(crate) trait FromReader {
    /// 節を読み込む。
    ///
    /// # 引数
    ///
    /// * `reader` - ファイルリーダ
    ///
    /// # 戻り値
    ///
    /// 節
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self>
    where
        Self: Sized;
}

pub(crate) trait TemplateFromReader<T> {
    /// テンプレートを読み込む。
    ///
    /// # 引数
    ///
    /// * `reader` - ファイルリーダ
    /// * `template_number` - テンプレート番号
    ///
    /// # 戻り値
    ///
    /// テンプレート
    fn from_reader(reader: &mut FileReader, template_number: T) -> ReaderResult<Self>
    where
        Self: Sized;
}

pub(crate) trait TemplateFromReaderWithSize<T> {
    /// テンプレートを読み込む。
    ///
    /// # 引数
    ///
    /// * `reader` - ファイルリーダ
    /// * `template_number` - テンプレート番号
    /// * `template_bytes` - テンプレートのバイト数
    ///
    /// # 戻り値
    ///
    /// テンプレート
    fn from_reader(
        reader: &mut FileReader,
        template_number: T,
        template_bytes: usize,
    ) -> ReaderResult<Self>
    where
        Self: Sized;
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
