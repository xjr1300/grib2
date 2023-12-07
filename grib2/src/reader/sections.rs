use std::io::{Read, Seek};

use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time};

use super::{FileReader, ReaderError, ReaderResult};
use macros::{Getter, SectionDebugInfo, TemplateDebugInfo, TemplateGetter};

/// 第0節:GRIB版番号
const EDITION_NUMBER: u8 = 2;

/// 第1節:節の長さ（バイト）
const SECTION1_BYTES: u32 = 21;

/// 第3節:格子系定義テンプレート番号
const LAT_LON_GRID_DEFINITION_TEMPLATE_NUMBER: u16 = 0; // 緯度・経度格子

/// 第４節:プロダクト定義テンプレート番号
const DEFAULT_PRODUCT_DEFINITION_TEMPLATE_NUMBER: u16 = 0; // デフォルト
const PROCESSED_PRODUCT_DEFINITION_TEMPLATE_NUMBER: u16 = 50000; // 他のプロダクトを元に加工・処理されたプロダクト
const RADAR_PRODUCT_DEFINITION_TEMPLATE_NUMBER: u16 = 50008; // レーダーなどに基づく解析プロダクト
const RADAR_FORECAST_PRODUCT_DEFINITION_TEMPLATE_NUMBER: u16 = 50009; // レーダーなどに基づく予測プロダクト

/// 第5節:資料表現テンプレート番号
const RUN_LENGTH_DATA_REPRESENTATION_TEMPLATE_NUMBER: u16 = 200; // ランレングス圧縮

/// 第6節:節の長さ（バイト）
const SECTION6_BYTES: u32 = 6;

/// 第8節:終端のマーカー
const SECTION8_END_MARKER: &str = "7777";

#[derive(Debug, Clone, Getter, SectionDebugInfo)]
#[section(number = 0, name = "指示節")]
pub struct Section0 {
    #[getter(ret = "ref", rty = "&str")]
    #[debug_info(name = "GRIB")]
    grib: String,
    #[getter(ret = "val")]
    #[debug_info(name = "資料分野")]
    discipline: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "保留")]
    reserved: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "GRIB版番号")]
    edition_number: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "GRIB報全体のバイト数", fmt = "0x{:08X}")]
    total_length: usize,
}

#[derive(Debug, Clone, Copy, Getter, SectionDebugInfo)]
#[section(number = 1, name = "識別節")]
pub struct Section1 {
    #[getter(ret = "val")]
    #[debug_info(name = "節の長さ", fmt = "0x{:04X}")]
    section_bytes: usize,
    #[getter(ret = "val")]
    #[debug_info(name = "作成中枢の識別")]
    center: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "作成副中枢")]
    sub_center: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "GRIBマスター表バージョン番号")]
    table_version: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "GRIB地域表バージョン番号")]
    local_table_version: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "参照時刻の意味")]
    significance_of_reference_time: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "資料の参照時刻(UTC)")]
    referenced_at: OffsetDateTime,
    #[getter(ret = "val")]
    #[debug_info(name = "作成ステータス")]
    production_status_of_processed_data: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "資料の種類")]
    type_of_processed_data: u8,
}

#[derive(Debug, Clone, Copy, SectionDebugInfo)]
#[section(number = 2, name = "地域使用節")]
pub struct Section2;

#[derive(Debug, Clone, Copy, Getter, SectionDebugInfo)]
#[section(number = 3, name = "格子系定義節")]
pub struct Section3<T> {
    #[getter(ret = "val")]
    #[debug_info(name = "節の長さ", fmt = "0x{:04X}")]
    section_bytes: usize,
    #[getter(ret = "val")]
    #[debug_info(name = "格子系定義の出典")]
    source_of_grid_definition: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "資料点数")]
    number_of_data_points: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "格子点数を定義するリストのオクテット数")]
    number_of_octets_for_number_of_points: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "格子点数を定義するリストの説明")]
    interpretation_of_number_of_points: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "格子系定義テンプレート番号")]
    grid_definition_template_number: u16,
    /// テンプレート3
    #[debug_template]
    template3: T,
}

/// テンプレート3.0
#[derive(Debug, Clone, Copy, TemplateGetter, TemplateDebugInfo)]
#[template_getter(section = "Section3", member = "template3")]
pub struct Template3_0 {
    #[getter(ret = "val")]
    #[debug_info(name = "地球の形状")]
    shape_of_earth: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "地球球体の半径の尺度因子")]
    scale_factor_of_radius_of_spherical_earth: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "地球球体の尺度付き半径")]
    scaled_value_of_radius_of_spherical_earth: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "地球回転楕円体の長軸の尺度因子")]
    scale_factor_of_earth_major_axis: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "地球回転楕円体の長軸の尺度付きの長さ")]
    scaled_value_of_earth_major_axis: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "地球回転楕円体の短軸の尺度因子")]
    scale_factor_of_earth_minor_axis: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "地球回転楕円体の短軸の尺度付きの長さ")]
    scaled_value_of_earth_minor_axis: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "緯線に沿った格子点数")]
    number_of_along_lat_points: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "経線に沿った格子点数")]
    number_of_along_lon_points: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "原作成領域の基本角")]
    basic_angle_of_initial_product_domain: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "端点の経度及び緯度並びに方向増分の定義に使われる基本角の細分")]
    subdivisions_of_basic_angle: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "最初の格子点の緯度（10e-6度単位）")]
    lat_of_first_grid_point: u32,
    #[debug_info(name = "最初の格子点の経度（10e-6度単位）")]
    #[getter(ret = "val")]
    lon_of_first_grid_point: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "分解能及び成分フラグ")]
    resolution_and_component_flags: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "最後の格子点の緯度（10e-6度単位）")]
    lat_of_last_grid_point: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "最後の格子点の経度（10e-6度単位）")]
    lon_of_last_grid_point: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "i方向（経度方向）の増分（10e-6度単位）")]
    i_direction_increment: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "j方向（緯度方向）の増分（10e-6度単位）")]
    j_direction_increment: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "走査モード")]
    scanning_mode: u8,
}

#[derive(Debug, Clone, Copy, Getter, SectionDebugInfo)]
#[section(number = 4, name = "プロダクト定義節")]
pub struct Section4<T> {
    #[getter(ret = "val")]
    #[debug_info(name = "節の長さ")]
    section_bytes: usize,
    #[getter(ret = "val")]
    #[debug_info(name = "テンプレート直後の座標値の数")]
    number_of_after_template_points: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "プロダクト定義テンプレート番号")]
    product_definition_template_number: u16,
    /// テンプレート4
    #[debug_template]
    template4: T,
}

/// テンプレート4.0
#[derive(Debug, Clone, Copy, TemplateGetter, TemplateDebugInfo)]
#[template_getter(section = "Section4", member = "template4")]
pub struct Template4_0 {
    #[getter(ret = "val")]
    #[debug_info(name = "パラメータカテゴリー")]
    parameter_category: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "パラメータ番号")]
    parameter_number: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "作成処理の種類")]
    type_of_generating_process: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "背景作成処理識別符")]
    background_process: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "予報の作成処理識別符")]
    generating_process_identifier: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "観測資料の参照時刻からの締切時間（時）")]
    hours_after_data_cutoff: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "観測資料の参照時刻からの締切時間（分）")]
    minutes_after_data_cutoff: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "期間の単位の指示符")]
    indicator_of_unit_of_time_range: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "予報時間")]
    forecast_time: i32,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の種類")]
    type_of_first_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の尺度因子")]
    scale_factor_of_first_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の尺度付きの値")]
    scaled_value_of_first_fixed_surface: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の種類")]
    type_of_second_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の尺度因子")]
    scale_factor_of_second_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の尺度付きの値")]
    scaled_value_of_second_fixed_surface: u32,
}

/// テンプレート4.50000
#[derive(Debug, Clone, Copy, TemplateGetter, TemplateDebugInfo)]
#[template_getter(section = "Section4", member = "template4")]
pub struct Template4_50000 {
    #[getter(ret = "val")]
    #[debug_info(name = "パラメータカテゴリー")]
    parameter_category: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "パラメータ番号")]
    parameter_number: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "作成処理の種類")]
    type_of_generating_process: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "背景作成処理識別符")]
    background_process: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "解析又は予報の作成処理識別符")]
    generating_process_identifier: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "観測資料の参照時刻からの締切時間（時）")]
    hours_after_data_cutoff: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "観測資料の参照時刻からの締切時間（分）")]
    minutes_after_data_cutoff: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "期間の単位の指示符")]
    indicator_of_unit_of_time_range: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "予報時間")]
    forecast_time: i32,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の種類")]
    type_of_first_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の尺度因子")]
    scale_factor_of_first_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の尺度付きの値")]
    scaled_value_of_first_fixed_surface: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の種類")]
    type_of_second_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の尺度因子")]
    scale_factor_of_second_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の尺度付きの値")]
    scaled_value_of_second_fixed_surface: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "資料作成に用いた関連資料の名称1")]
    source_document1: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "上記関連資料の解析時刻と参照時刻との差（時）1")]
    hours_from_source_document1: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "上記関連資料の解析時刻と参照時刻との差（分）1")]
    minutes_from_source_document1: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "資料作成に用いた関連資料の名称2")]
    source_document2: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "上記関連資料の解析時刻と参照時刻との差（時）2")]
    hours_from_source_document2: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "上記関連資料の解析時刻と参照時刻との差（分）2")]
    minutes_from_source_document2: u8,
}

/// テンプレート4.50008
#[derive(Debug, Clone, Copy, TemplateGetter, TemplateDebugInfo)]
#[template_getter(section = "Section4", member = "template4")]
pub struct Template4_50008 {
    #[getter(ret = "val")]
    #[debug_info(name = "パラメータカテゴリー")]
    parameter_category: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "パラメータ番号")]
    parameter_number: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "作成処理の種類")]
    type_of_generating_process: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "背景作成処理識別符")]
    background_process: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "予報の作成処理識別符")]
    generating_process_identifier: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "観測資料の参照時刻からの締切時間（時）")]
    hours_after_data_cutoff: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "観測資料の参照時刻からの締切時間（分）")]
    minutes_after_data_cutoff: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "期間の単位の指示符")]
    indicator_of_unit_of_time_range: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "予報時間")]
    forecast_time: i32,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の種類")]
    type_of_first_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の尺度因子")]
    scale_factor_of_first_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の尺度付きの値")]
    scaled_value_of_first_fixed_surface: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の種類")]
    type_of_second_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の尺度因子")]
    scale_factor_of_second_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の尺度付きの値")]
    scaled_value_of_second_fixed_surface: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "全時間間隔の終了時(UTC)")]
    end_of_all_time_intervals: OffsetDateTime,
    #[getter(ret = "val")]
    #[debug_info(name = "統計を算出するために使用した時間間隔を記述する期間の仕様の数")]
    number_of_time_range_specs: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理における欠測資料の総数")]
    number_of_missing_values: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理の種類")]
    type_of_stat_proc: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理の時間増分の種類")]
    type_of_stat_proc_time_increment: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理の時間の単位の指示符")]
    stat_proc_time_unit: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理した時間の長さ")]
    stat_proc_time_length: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "連続的な資料場間の増分に関する時間の単位の指示符")]
    successive_time_unit: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "連続的な資料場間の時間の増分")]
    successive_time_increment: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "レーダー等運用情報その1", fmt = "0x{:08X}")]
    radar_info1: u64,
    #[getter(ret = "val")]
    #[debug_info(name = "レーダー等運用情報その2", fmt = "0x{:08X}")]
    radar_info2: u64,
    #[getter(ret = "val")]
    #[debug_info(name = "雨量計運用情報", fmt = "0x{:08X}")]
    rain_gauge_info: u64,
}

/// テンプレート4.50009
#[derive(Debug, Clone, TemplateGetter, TemplateDebugInfo)]
#[template_getter(section = "Section4", member = "template4")]
pub struct Template4_50009 {
    #[getter(ret = "val")]
    #[debug_info(name = "パラメータカテゴリー")]
    parameter_category: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "パラメータ番号")]
    parameter_number: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "作成処理の種類")]
    type_of_generating_process: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "背景作成処理識別符")]
    background_process: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "予報の作成処理識別符")]
    generating_process_identifier: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "観測資料の参照時刻からの締切時間（時）")]
    hours_after_data_cutoff: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "観測資料の参照時刻からの締切時間（分）")]
    minutes_after_data_cutoff: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "期間の単位の指示符")]
    indicator_of_unit_of_time_range: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "予報時間")]
    forecast_time: i32,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の種類")]
    type_of_first_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の尺度因子")]
    scale_factor_of_first_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第一固定面の尺度付きの値")]
    scaled_value_of_first_fixed_surface: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の種類")]
    type_of_second_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の尺度因子")]
    scale_factor_of_second_fixed_surface: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "第二固定面の尺度付きの値")]
    scaled_value_of_second_fixed_surface: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "全時間間隔の終了時(UTC)")]
    end_of_all_time_intervals: OffsetDateTime,
    #[getter(ret = "val")]
    #[debug_info(name = "統計を算出するために使用した時間間隔を記述する期間の仕様の数")]
    number_of_time_range_specs: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理における欠測資料の総数")]
    number_of_missing_values: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理の種類")]
    type_of_stat_proc: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理の時間増分の種類")]
    type_of_stat_proc_time_increment: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理の時間の単位の指示符")]
    stat_proc_time_unit: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "統計処理した時間の長さ")]
    stat_proc_time_length: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "連続的な資料場間の増分に関する時間の単位の指示符")]
    successive_time_unit: u8,
    #[getter(ret = "val")]
    #[debug_info(name = "連続的な資料場間の時間の増分")]
    successive_time_increment: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "レーダー等運用情報その1", fmt = "0x{:08X}")]
    radar_info1: u64,
    #[getter(ret = "val")]
    #[debug_info(name = "レーダー等運用情報その2", fmt = "0x{:08X}")]
    radar_info2: u64,
    #[getter(ret = "val")]
    #[debug_info(name = "雨量計運用情報", fmt = "0x{:08X}")]
    rain_gauge_info: u64,
    #[getter(ret = "val")]
    #[debug_info(name = "メソモデル予想値の結合比率の計算領域数")]
    number_of_calculation_areas: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "メソモデル予想値の結合比率の尺度因子")]
    scale_factor_of_combined_ratio: u8,
    #[getter(ret = "ref", rty = "&[u16]")]
    #[debug_info(
        name = "各領域のメソモデル予想値の結合比率",
        data_type = "serial",
        header = "結合比率{}",
        start = 1,
        fmt = "{}"
    )]
    combined_ratios_of_forecast_areas: Vec<u16>,
}

#[derive(Debug, Clone, Copy, Getter, SectionDebugInfo)]
#[section(number = 5, name = "資料表現節")]
pub struct Section5<T> {
    #[getter(ret = "val")]
    #[debug_info(name = "節の長さ", fmt = "0x{:04X}")]
    section_bytes: usize,
    #[getter(ret = "val")]
    #[debug_info(name = "全資料点の数")]
    number_of_values: u32,
    #[getter(ret = "val")]
    #[debug_info(name = "資料表現テンプレート番号")]
    data_representation_template_number: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "1データのビット数")]
    bits_per_value: u8,
    /// テンプレート5
    #[debug_template]
    template5: T,
}

/// テンプレート5.200
#[derive(Debug, Clone, TemplateGetter, TemplateDebugInfo)]
#[template_getter(section = "Section5", member = "template5")]
pub struct Template5_200 {
    #[getter(ret = "val")]
    #[debug_info(name = "今回の圧縮に用いたレベルの最大値")]
    max_level_value: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "データの取り得るレベルの最大値")]
    number_of_level_values: u16,
    #[getter(ret = "val")]
    #[debug_info(name = "データ代表値の尺度因子")]
    decimal_scale_factor: u8,
    /// レベル値と物理値(mm/h)の対応を格納するコレクション
    #[getter(ret = "ref", rty = "&[u16]")]
    #[debug_info(
        name = "レベルmに対応するデータ代表値",
        data_type = "serial",
        header = "レベル{}",
        start = 1,
        fmt = "{}"
    )]
    level_values: Vec<u16>,
}

#[derive(Debug, Clone, Copy, Getter, SectionDebugInfo)]
#[section(number = 6, name = "ビットマップ節")]
pub struct Section6 {
    #[getter(ret = "val")]
    #[debug_info(name = "節の長さ", fmt = "0x{:04X}")]
    section_bytes: usize,
    #[getter(ret = "val")]
    #[debug_info(name = "ビットマップ指示符")]
    bitmap_indicator: u8,
}

#[derive(Debug, Clone, Copy, Getter, SectionDebugInfo)]
#[section(number = 7, name = "資料節")]
pub struct Section7<T> {
    #[getter(ret = "val")]
    #[debug_info(name = "節の長さ", fmt = "0x{:04X}")]
    section_bytes: usize,
    #[debug_template]
    template7: T,
}

/// テンプレート7.200
#[derive(Debug, Clone, Copy, TemplateGetter, TemplateDebugInfo)]
#[template_getter(section = "Section7", member = "template7")]
pub struct Template7_200 {
    #[getter(ret = "val")]
    #[debug_info(name = "ランレングス圧縮符号列の開始位置", fmt = "0x{:08X}")]
    run_length_position: usize,
    #[getter(ret = "val")]
    #[debug_info(name = "ランレングス圧縮符号のバイト数", fmt = "0x{:08X}")]
    run_length_bytes: usize,
}

/// 第８節:終端節
#[derive(Debug, Clone, Getter, SectionDebugInfo)]
#[section(number = 8, name = "終端節")]
pub struct Section8 {
    /// 終端のマーカー
    #[getter(ret = "ref", rty = "&str")]
    #[debug_info(name = "終了マーカー")]
    end_marker: String,
}

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
        let grib = validate_str(reader, "第0節:GRIB", 4, "GRIB")?;
        // 保留: 2バイト
        let reserved = read_u16(reader, "第0節:保留")?;
        // 資料分野: 1バイト
        let discipline = read_u8(reader, "第0節:資料分野")?;
        // GRIB版番号: 1バイト
        let edition_number = validate_u8(reader, EDITION_NUMBER, "第0節:GRIB版番号")?;
        // GRIB報全体の長さ: 8バイト
        let total_length = read_u64(reader, "第0節:GRIB報全体の長さ")? as usize;

        Ok(Self {
            grib,
            discipline,
            reserved,
            edition_number,
            total_length,
        })
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
}

impl<T> FromReader for Section3<T>
where
    T: TemplateFromReader<u16>,
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
        let template = T::from_reader(reader, grid_definition_template_number)?;

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
        let scale_factor_of_radius_of_spherical_earth =
            read_u8(reader, "第3節:地球球体の半径の尺度因子")?;
        // 地球球体の尺度付き半径: 4バイト
        let scaled_value_of_radius_of_spherical_earth =
            read_u32(reader, "第3節:地球球体の尺度付き半径")?;
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
        let subdivisions_of_basic_angle =
            read_u32(reader, "第3節:端点の経度及び緯度並びに方向増分の定義")?;
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
            scale_factor_of_radius_of_spherical_earth,
            scaled_value_of_radius_of_spherical_earth,
            scale_factor_of_earth_major_axis,
            scaled_value_of_earth_major_axis,
            scale_factor_of_earth_minor_axis,
            scaled_value_of_earth_minor_axis,
            number_of_along_lat_points,
            number_of_along_lon_points,
            basic_angle_of_initial_product_domain,
            subdivisions_of_basic_angle,
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

impl<T> FromReader for Section4<T>
where
    T: TemplateFromReader<u16>,
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
        // テンプレート4
        let template4 = T::from_reader(reader, product_definition_template_number)?;

        Ok(Self {
            section_bytes,
            number_of_after_template_points,
            product_definition_template_number,
            template4,
        })
    }
}

impl TemplateFromReader<u16> for Template4_0 {
    fn from_reader(reader: &mut FileReader, template_number: u16) -> ReaderResult<Self> {
        // プロダクト定義テンプレート番号を確認
        validate_template_number!(
            "第4節:プロダクト定義テンプレート番号",
            template_number,
            DEFAULT_PRODUCT_DEFINITION_TEMPLATE_NUMBER
        );
        // パラメータカテゴリー: 1バイト
        let parameter_category = read_u8(reader, "第4節:パラメータカテゴリー")?;
        // パラメータ番号: 1バイト
        let parameter_number = read_u8(reader, "第4節:パラメータ番号")?;
        // 作成処理の種類: 1バイト
        let type_of_generating_process = read_u8(reader, "第4節:作成処理の種類")?;
        // 背景作成処理識別符: 1バイト
        let background_process = read_u8(reader, "第4節:背景作成処理識別符")?;
        // 予報の作成処理識別符: 1バイト
        let generating_process_identifier = read_u8(reader, "第4節:予報の作成処理識別符")?;
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
        let scale_factor_of_first_fixed_surface = read_u8(reader, "第4節:第一固定面の尺度因子")?;
        // 第一固定面の尺度付きの値: 4バイト
        let scaled_value_of_first_fixed_surface =
            read_u32(reader, "第4節:第一固定面の尺度付きの値")?;
        // 第二固定面の種類: 1バイト
        let type_of_second_fixed_surface = read_u8(reader, "第4節:第二固定面の種類")?;
        // 第二固定面の尺度因子: 1バイト
        let scale_factor_of_second_fixed_surface = read_u8(reader, "第4節:第二固定面の尺度因子")?;
        // 第二固定面の尺度付きの値: 4バイト
        let scaled_value_of_second_fixed_surface =
            read_u32(reader, "第4節:第二固定面の尺度付きの値")?;

        Ok(Self {
            parameter_category,
            parameter_number,
            type_of_generating_process,
            background_process,
            generating_process_identifier,
            hours_after_data_cutoff,
            minutes_after_data_cutoff,
            indicator_of_unit_of_time_range,
            forecast_time,
            type_of_first_fixed_surface,
            scale_factor_of_first_fixed_surface,
            scaled_value_of_first_fixed_surface,
            type_of_second_fixed_surface,
            scale_factor_of_second_fixed_surface,
            scaled_value_of_second_fixed_surface,
        })
    }
}

impl TemplateFromReader<u16> for Template4_50000 {
    fn from_reader(reader: &mut FileReader, template_number: u16) -> ReaderResult<Self> {
        // プロダクト定義テンプレート番号を確認
        validate_template_number!(
            "第4節:プロダクト定義テンプレート番号",
            template_number,
            PROCESSED_PRODUCT_DEFINITION_TEMPLATE_NUMBER
        );
        // パラメータカテゴリー: 1バイト
        let parameter_category = read_u8(reader, "第4節:パラメータカテゴリー")?;
        // パラメータ番号: 1バイト
        let parameter_number = read_u8(reader, "第4節:パラメータ番号")?;
        // 作成処理の種類: 1バイト
        let type_of_generating_process = read_u8(reader, "第4節:作成処理の種類")?;
        // 背景作成処理識別符: 1バイト
        let background_process = read_u8(reader, "第4節:背景作成処理識別符")?;
        // 解析又は予報の作成処理識別符: 1バイト
        let generating_process_identifier = read_u8(reader, "第4節:解析又は予報の作成処理識別符")?;
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
        let scale_factor_of_first_fixed_surface = read_u8(reader, "第4節:第一固定面の尺度因子")?;
        // 第一固定面の尺度付きの値: 4バイト
        let scaled_value_of_first_fixed_surface =
            read_u32(reader, "第4節:第一固定面の尺度付きの値")?;
        // 第二固定面の種類: 1バイト
        let type_of_second_fixed_surface = read_u8(reader, "第4節:第二固定面の種類")?;
        // 第二固定面の尺度因子: 1バイト
        let scale_factor_of_second_fixed_surface = read_u8(reader, "第4節:第二固定面の尺度因子")?;
        // 第二固定面の尺度付きの値: 4バイト
        let scaled_value_of_second_fixed_surface =
            read_u32(reader, "第4節:第二固定面の尺度付きの値")?;
        // 資料作成に用いた関連資料の名称1: 1バイト
        let source_document1 = read_u8(reader, "第4節:資料作成に用いた関連資料の名称1")?;
        // 上記関連資料の解析時刻と参照時刻との差（時）1: 2バイト
        let hours_from_source_document1 =
            read_u16(reader, "第4節:記関連資料の解析時刻と参照時刻との差（時）1")?;
        // 上記関連資料の解析時刻と参照時刻との差（分）1: 1バイト
        let minutes_from_source_document1 =
            read_u8(reader, "第4節:記関連資料の解析時刻と参照時刻との差（分）1")?;
        // 資料作成に用いた関連資料の名称2: 1バイト
        let source_document2 = read_u8(reader, "第4節:資料作成に用いた関連資料の名称2")?;
        // 上記関連資料の解析時刻と参照時刻との差（時）2: 2バイト
        let hours_from_source_document2 =
            read_u16(reader, "第4節:記関連資料の解析時刻と参照時刻との差（時）2")?;
        // 上記関連資料の解析時刻と参照時刻との差（分）2: 1バイト
        let minutes_from_source_document2 =
            read_u8(reader, "第4節:記関連資料の解析時刻と参照時刻との差（分）2")?;

        Ok(Self {
            parameter_category,
            parameter_number,
            type_of_generating_process,
            background_process,
            generating_process_identifier,
            hours_after_data_cutoff,
            minutes_after_data_cutoff,
            indicator_of_unit_of_time_range,
            forecast_time,
            type_of_first_fixed_surface,
            scale_factor_of_first_fixed_surface,
            scaled_value_of_first_fixed_surface,
            type_of_second_fixed_surface,
            scale_factor_of_second_fixed_surface,
            scaled_value_of_second_fixed_surface,
            source_document1,
            hours_from_source_document1,
            minutes_from_source_document1,
            source_document2,
            hours_from_source_document2,
            minutes_from_source_document2,
        })
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
        // パラメータカテゴリー: 1バイト
        let parameter_category = read_u8(reader, "第4節:パラメータカテゴリー")?;
        // パラメータ番号: 1バイト
        let parameter_number = read_u8(reader, "第4節:パラメータ番号")?;
        // 作成処理の種類: 1バイト
        let type_of_generating_process = read_u8(reader, "第4節:作成処理の種類")?;
        // 背景作成処理識別符: 1バイト
        let background_process = read_u8(reader, "第4節:背景作成処理識別符")?;
        // 予報の作成処理識別符: 1バイト
        let generating_process_identifier = read_u8(reader, "第4節:予報の作成処理識別符")?;
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
        let scale_factor_of_first_fixed_surface = read_u8(reader, "第4節:第一固定面の尺度因子")?;
        // 第一固定面の尺度付きの値: 4バイト
        let scaled_value_of_first_fixed_surface =
            read_u32(reader, "第4節:第一固定面の尺度付きの値")?;
        // 第二固定面の種類: 1バイト
        let type_of_second_fixed_surface = read_u8(reader, "第4節:第二固定面の種類")?;
        // 第二固定面の尺度因子: 1バイト
        let scale_factor_of_second_fixed_surface = read_u8(reader, "第4節:第二固定面の尺度因子")?;
        // 第二固定面の尺度付きの値: 4バイト
        let scaled_value_of_second_fixed_surface =
            read_u32(reader, "第4節:第二固定面の尺度付きの値")?;
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
            parameter_category,
            parameter_number,
            type_of_generating_process,
            background_process,
            generating_process_identifier,
            hours_after_data_cutoff,
            minutes_after_data_cutoff,
            indicator_of_unit_of_time_range,
            forecast_time,
            type_of_first_fixed_surface,
            scale_factor_of_first_fixed_surface,
            scaled_value_of_first_fixed_surface,
            type_of_second_fixed_surface,
            scale_factor_of_second_fixed_surface,
            scaled_value_of_second_fixed_surface,
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

impl TemplateFromReader<u16> for Template4_50009 {
    fn from_reader(reader: &mut FileReader, template_number: u16) -> ReaderResult<Self> {
        // プロダクト定義テンプレート番号を確認
        validate_template_number!(
            "第4節:プロダクト定義テンプレート番号",
            template_number,
            RADAR_FORECAST_PRODUCT_DEFINITION_TEMPLATE_NUMBER
        );
        // パラメータカテゴリー: 1バイト
        let parameter_category = read_u8(reader, "第4節:パラメータカテゴリー")?;
        // パラメータ番号: 1バイト
        let parameter_number = read_u8(reader, "第4節:パラメータ番号")?;
        // 作成処理の種類: 1バイト
        let type_of_generating_process = read_u8(reader, "第4節:作成処理の種類")?;
        // 背景作成処理識別符: 1バイト
        let background_process = read_u8(reader, "第4節:背景作成処理識別符")?;
        // 予報の作成処理識別符: 1バイト
        let generating_process_identifier = read_u8(reader, "第4節:予報の作成処理識別符")?;
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
        let scale_factor_of_first_fixed_surface = read_u8(reader, "第4節:第一固定面の尺度因子")?;
        // 第一固定面の尺度付きの値: 4バイト
        let scaled_value_of_first_fixed_surface =
            read_u32(reader, "第4節:第一固定面の尺度付きの値")?;
        // 第二固定面の種類: 1バイト
        let type_of_second_fixed_surface = read_u8(reader, "第4節:第二固定面の種類")?;
        // 第二固定面の尺度因子: 1バイト
        let scale_factor_of_second_fixed_surface = read_u8(reader, "第4節:第二固定面の尺度因子")?;
        // 第二固定面の尺度付きの値: 4バイト
        let scaled_value_of_second_fixed_surface =
            read_u32(reader, "第4節:第二固定面の尺度付きの値")?;
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
        // メソモデル予想値の結合比率の計算領域数
        let number_of_calculation_areas =
            read_u16(reader, "メソモデル予想値の結合比率の計算領域数")?;
        // メソモデル予想値の結合比率の尺度因子
        let scale_factor_of_combined_ratio =
            read_u8(reader, "メソモデル予想値の結合比率の尺度因子")?;
        // 各領域のメソモデル予想値の結合比率
        let mut combined_ratios_of_forecast_areas = vec![];
        for _ in 0..number_of_calculation_areas {
            combined_ratios_of_forecast_areas
                .push(read_u16(reader, "各領域のメソモデル予想値の結合比率")?);
        }

        Ok(Self {
            parameter_category,
            parameter_number,
            type_of_generating_process,
            background_process,
            generating_process_identifier,
            hours_after_data_cutoff,
            minutes_after_data_cutoff,
            indicator_of_unit_of_time_range,
            forecast_time,
            type_of_first_fixed_surface,
            scale_factor_of_first_fixed_surface,
            scaled_value_of_first_fixed_surface,
            type_of_second_fixed_surface,
            scale_factor_of_second_fixed_surface,
            scaled_value_of_second_fixed_surface,
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
            number_of_calculation_areas,
            scale_factor_of_combined_ratio,
            combined_ratios_of_forecast_areas,
        })
    }
}

impl<T> FromReader for Section5<T>
where
    T: TemplateFromReaderWithSize<u16>,
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
            T::from_reader(reader, data_representation_template_number, template_bytes)?;

        Ok(Self {
            section_bytes,
            number_of_values,
            data_representation_template_number,
            bits_per_value,
            template5,
        })
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

impl<T> FromReader for Section7<T>
where
    T: TemplateFromReaderWithSize<u16>,
{
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // 節の長さ: 4バイト
        let section_bytes = read_u32(reader, "第7節:節の長さ")? as usize;
        // 節番号: 1バイト
        validate_u8(reader, 7, "第7節:節番号")?;
        // テンプレート7
        let template_bytes = section_bytes - (4 + 1);
        let template7 = T::from_reader(
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

/// 土壌雨量指数の第4節から第7節
pub struct SwiSections {
    section4: Section4_0,
    section5: Section5_200,
    section6: Section6,
    section7: Section7_200,
}

impl SwiSections {
    pub(crate) fn from_reader(reader: &mut FileReader) -> ReaderResult<SwiSections> {
        let section4 = Section4_0::from_reader(reader)?;
        let section5 = Section5_200::from_reader(reader)?;
        let section6 = Section6::from_reader(reader)?;
        let section7 = Section7_200::from_reader(reader)?;

        Ok(SwiSections {
            section4,
            section5,
            section6,
            section7,
        })
    }

    /// 第4節:プロダクト定義節を返す。
    ///
    /// # 戻り値
    ///
    /// 第4節:プロダクト定義節
    pub fn section4(&self) -> &Section4_0 {
        &self.section4
    }

    /// 第5節:資料表現節を返す。
    ///
    /// # 戻り値
    ///
    /// 第5節:資料表現節
    pub fn section5(&self) -> &Section5_200 {
        &self.section5
    }

    /// 第6節:ビットマップ節を返す。
    ///
    /// # 戻り値
    ///
    /// 第6節:ビットマップ節
    pub fn section6(&self) -> &Section6 {
        &self.section6
    }

    /// 第7節:資料節を返す。
    ///
    /// # 戻り値
    ///
    /// 第7節:資料節
    pub fn section7(&self) -> &Section7_200 {
        &self.section7
    }

    /// 第4節から第7節を出力する。
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.section4.debug_info(writer)?;
        writeln!(writer)?;
        self.section5.debug_info(writer)?;
        writeln!(writer)?;
        self.section6.debug_info(writer)?;
        writeln!(writer)?;
        self.section7.debug_info(writer)?;

        Ok(())
    }
}

impl FromReader for Section8 {
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        // 第8節:終端マーカー
        let end_marker = read_str(reader, 4);
        match end_marker {
            Ok(end_marker) => {
                if end_marker == SECTION8_END_MARKER {
                    Ok(Self { end_marker })
                } else {
                    Err(ReaderError::Unexpected(
                        format!(
                            "第8節の終了が不正です。ファイルを正確に読み込めなかった可能性があります。expected: {}, actual: {}",
                            SECTION8_END_MARKER, end_marker
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

fn read_i32(reader: &mut FileReader, name: &str) -> ReaderResult<i32> {
    let expected_bytes = std::mem::size_of::<i32>();
    let mut buf = vec![0_u8; expected_bytes];
    reader.read_exact(&mut buf).map_err(|_| {
        ReaderError::ReadError(format!("{}の読み込みに失敗しました。", name).into())
    })?;
    // 最上位ビットを確認(0であれば正の数、1であれば負の数)
    let sign = if buf[0] & 0x80 == 0 { 1 } else { -1 };
    // 最上位ビットを0にした結果をデコード
    buf[0] &= 0x7F;

    Ok(<i32>::from_be_bytes(buf.try_into().unwrap()) * sign)
}

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

fn read_datetime(reader: &mut FileReader, name: &str) -> ReaderResult<OffsetDateTime> {
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
                "{}:{}年{}月{}日を日付に変換できませんでした。",
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

    Ok(PrimitiveDateTime::new(date, time).assume_utc())
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

pub type Section3_0 = Section3<Template3_0>;
pub type Section4_0 = Section4<Template4_0>;
pub type Section4_50000 = Section4<Template4_50000>;
pub type Section4_50008 = Section4<Template4_50008>;
pub type Section4_50009 = Section4<Template4_50009>;
pub type Section5_200 = Section5<Template5_200>;
pub type Section7_200 = Section7<Template7_200>;
