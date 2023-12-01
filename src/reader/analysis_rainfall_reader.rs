use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::str;

use num_format::{Locale, ToFormattedString};
use time::{Date, Month, PrimitiveDateTime, Time};

use super::value_iter::Grib2ValueIter;
use super::{FileReader, ReaderError, ReaderResult};

/// 1kmメッシュ解析雨量リーダー
pub struct AnalysisRainfallReader<P>
where
    P: AsRef<Path>,
{
    path: P,
    inner: Inner,
}

impl<P> AnalysisRainfallReader<P>
where
    P: AsRef<Path>,
{
    /// ファイルパスを受け取り`Grib2Reader`を構築する。
    ///
    /// # 引数
    ///
    /// * `path` - GRIB2形式のファイルのパス
    ///
    /// # 戻り値
    ///
    /// `Grib2Reader`
    pub fn new(path: P) -> ReaderResult<Self> {
        let file =
            File::open(path.as_ref()).map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let reader = BufReader::new(file);
        let mut inner = Inner::default();
        inner.read_to_end(reader)?;
        if inner.number_of_data_points.unwrap() != inner.number_of_values.unwrap() {
            return Err(ReaderError::Unexpected(
                format!(
                    "第3節に記録されている資料点数({})と第5節に記録されている全資料点({})が一致しません。",
                    inner.number_of_data_points.unwrap().to_formatted_string(&Locale::ja),
                    inner.number_of_values.unwrap().to_formatted_string(&Locale::ja),
                ).into(),
            ));
        }

        Ok(AnalysisRainfallReader { path, inner })
    }

    /// 資料分野を返す。
    ///
    /// # 戻り値
    ///
    /// 資料分野
    pub fn discipline(&self) -> u8 {
        self.inner.discipline.unwrap()
    }

    /// GRIB版番号を返す。
    ///
    /// # 戻り値
    ///
    /// GRIB版番号
    pub fn edition_number(&self) -> u8 {
        self.inner.edition_number.unwrap()
    }

    /// GRIB報全体のバイト数を返す。
    ///
    ///
    /// # 戻り値
    ///
    /// GRIB報全体のバイト数
    pub fn total_length(&self) -> usize {
        self.inner.total_length.unwrap()
    }

    /// 作成中枢の識別を返す。
    ///
    /// # 戻り値
    ///
    /// 作成中枢の識別
    pub fn center(&self) -> u16 {
        self.inner.center.unwrap()
    }

    /// 作成副中枢を返す。
    ///
    /// # 戻り値
    ///
    /// 作成副中枢
    pub fn sub_center(&self) -> u16 {
        self.inner.sub_center.unwrap()
    }

    /// GRIBマスター表バージョン番号を返す。
    ///
    /// # 戻り値
    ///
    /// GRIBマスター表バージョン番号
    pub fn table_version(&self) -> u8 {
        self.inner.table_version.unwrap()
    }

    /// GRIB地域表バージョン番号を返す。
    ///
    /// # 戻り値
    ///
    /// GRIB地域表バージョン番号
    pub fn local_table_version(&self) -> u8 {
        self.inner.local_table_version.unwrap()
    }

    /// 参照時刻の意味を返す。
    ///
    /// # 戻り値
    ///
    /// 参照時刻の意味
    pub fn significance_of_reference_time(&self) -> u8 {
        self.inner.significance_of_reference_time.unwrap()
    }

    /// 資料の参照時刻を返す。
    ///
    /// # 戻り値
    ///
    /// 資料の参照時刻
    pub fn referenced_at(&self) -> PrimitiveDateTime {
        self.inner.referenced_at.unwrap()
    }

    /// 作成ステータスを返す。
    ///
    /// # 戻り値
    ///
    /// 作成ステータス
    pub fn production_status_of_processed_data(&self) -> u8 {
        self.inner.production_status_of_processed_data.unwrap()
    }

    /// 資料の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 資料の種類
    pub fn type_of_processed_data(&self) -> u8 {
        self.inner.type_of_processed_data.unwrap()
    }

    /// 格子系定義の出典を返す。
    ///
    /// # 戻り値
    ///
    /// 格子系定義の出典
    pub fn source_of_grid_definition(&self) -> u8 {
        self.inner.source_of_grid_definition.unwrap()
    }

    /// 第3節に記録されている資料点数を返す。
    ///
    /// # 戻り値
    ///
    /// 第3節に記録されている資料点数
    pub fn number_of_data_points(&self) -> u32 {
        self.inner.number_of_data_points.unwrap()
    }

    /// 格子点数を定義するリストのオクテット数を返す。
    ///
    /// # 戻り値
    ///
    /// 格子点数を定義するリストのオクテット数
    pub fn number_of_octets_for_number_of_points(&self) -> u8 {
        self.inner.number_of_octets_for_number_of_points.unwrap()
    }

    /// 格子点数を定義するリストの説明を返す。
    ///
    /// # 戻り値
    ///
    /// 格子点数を定義するリストの説明
    pub fn interpretation_of_number_of_points(&self) -> u8 {
        self.inner.interpretation_of_number_of_points.unwrap()
    }

    /// 格子系定義テンプレート番号を返す。
    ///
    /// # 戻り値
    ///
    /// 格子系定義テンプレート番号
    pub fn grid_definition_template_number(&self) -> u16 {
        self.inner.grid_definition_template_number.unwrap()
    }

    /// 地球の形状を返す。
    ///
    /// # 戻り値
    ///
    /// 地球の形状
    pub fn shape_of_earth(&self) -> u8 {
        self.inner.shape_of_earth.unwrap()
    }

    /// 地球回転楕円体の長軸の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の長軸の尺度因子
    pub fn scale_factor_of_earth_major_axis(&self) -> u8 {
        self.inner.scale_factor_of_earth_major_axis.unwrap()
    }

    /// 地球回転楕円体の長軸の尺度付きの長さを返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の長軸の尺度付きの長さ
    pub fn scaled_value_of_earth_major_axis(&self) -> u32 {
        self.inner.scaled_value_of_earth_major_axis.unwrap()
    }

    /// 地球回転楕円体の短軸の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の短軸の尺度因子
    pub fn scale_factor_of_earth_minor_axis(&self) -> u8 {
        self.inner.scale_factor_of_earth_minor_axis.unwrap()
    }

    /// 地球回転楕円体の短軸の尺度付きの長さを返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の短軸の尺度付きの長さ
    pub fn scaled_value_of_earth_minor_axis(&self) -> u32 {
        self.inner.scaled_value_of_earth_minor_axis.unwrap()
    }

    /// 緯線に沿った格子点数を返す。
    ///
    /// # 戻り値
    ///
    /// 緯線に沿った格子点数
    pub fn number_of_along_lat_points(&self) -> u32 {
        self.inner.number_of_along_lat_points.unwrap()
    }

    /// 経線に沿った格子点数を返す。
    ///
    /// # 戻り値
    ///
    /// 経線に沿った格子点数
    pub fn number_of_along_lon_points(&self) -> u32 {
        self.inner.number_of_along_lon_points.unwrap()
    }

    /// 原作成領域の基本角を返す。
    ///
    /// # 戻り値
    ///
    /// 原作成領域の基本角
    pub fn basic_angle_of_initial_product_domain(&self) -> u32 {
        self.inner.basic_angle_of_initial_product_domain.unwrap()
    }

    /// 最初の格子点の緯度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最初の格子点の緯度（10e-6度単位）
    pub fn lat_of_first_grid_point(&self) -> u32 {
        self.inner.lat_of_first_grid_point.unwrap()
    }

    /// 最初の格子点の経度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最初の格子点の経度（10e-6度単位）
    pub fn lon_of_first_grid_point(&self) -> u32 {
        self.inner.lon_of_first_grid_point.unwrap()
    }

    /// 分解能及び成分フラグを返す。
    ///
    /// # 戻り値
    ///
    /// 分解能及び成分フラグ
    pub fn resolution_and_component_flags(&self) -> u8 {
        self.inner.resolution_and_component_flags.unwrap()
    }

    /// 最後の格子点の緯度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最後の格子点の緯度（10e-6度単位）
    pub fn lat_of_last_grid_point(&self) -> u32 {
        self.inner.lat_of_last_grid_point.unwrap()
    }

    /// 最後の格子点の経度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最後の格子点の経度（10e-6度単位）
    pub fn lon_of_last_grid_point(&self) -> u32 {
        self.inner.lon_of_last_grid_point.unwrap()
    }

    /// i方向（経度方向）の増分（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// i方向（経度方向）の増分（10e-6度単位）
    pub fn i_direction_increment(&self) -> u32 {
        self.inner.i_direction_increment.unwrap()
    }

    /// j方向（緯度方向）の増分（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// j方向（緯度方向）の増分（10e-6度単位）
    pub fn j_direction_increment(&self) -> u32 {
        self.inner.j_direction_increment.unwrap()
    }

    /// 走査モードを返す。
    ///
    /// # 戻り値
    ///
    /// 走査モード
    pub fn scanning_mode(&self) -> u8 {
        self.inner.scanning_mode.unwrap()
    }

    /// テンプレート直後の座標値の数を返す。
    ///
    /// # 戻り値
    ///
    /// テンプレート直後の座標値の数
    pub fn number_of_after_template_points(&self) -> u16 {
        self.inner.number_of_after_template_points.unwrap()
    }

    /// プロダクト定義テンプレート番号を返す。
    ///
    /// # 戻り値
    ///
    /// プロダクト定義テンプレート番号
    pub fn product_definition_template_number(&self) -> u16 {
        self.inner.product_definition_template_number.unwrap()
    }

    /// パラメータカテゴリーを返す。
    ///
    /// # 戻り値
    ///
    /// パラメータカテゴリー
    pub fn parameter_category(&self) -> u8 {
        self.inner.parameter_category.unwrap()
    }

    /// パラメータ番号を返す。
    ///
    /// # 戻り値
    ///
    /// パラメータ番号
    pub fn parameter_number(&self) -> u8 {
        self.inner.parameter_number.unwrap()
    }

    /// 作成処理の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 作成処理の種類
    pub fn type_of_generating_process(&self) -> u8 {
        self.inner.type_of_generating_process.unwrap()
    }

    /// 背景作成処理識別符を返す。
    ///
    /// # 戻り値
    ///
    /// 背景作成処理識別符
    pub fn background_process(&self) -> u8 {
        self.inner.background_process.unwrap()
    }

    /// 観測資料の参照時刻からの締切時間（時）を返す。
    ///
    /// # 戻り値
    ///
    /// 観測資料の参照時刻からの締切時間（時）
    pub fn hours_after_data_cutoff(&self) -> u16 {
        self.inner.hours_after_data_cutoff.unwrap()
    }

    /// 観測資料の参照時刻からの締切時間（分）を返す。
    ///
    /// # 戻り値
    ///
    /// 観測資料の参照時刻からの締切時間（分）
    pub fn minutes_after_data_cutoff(&self) -> u8 {
        self.inner.minutes_after_data_cutoff.unwrap()
    }

    /// 期間の単位の指示符を返す。
    ///
    /// # 戻り値
    ///
    /// 期間の単位の指示符
    pub fn indicator_of_unit_of_time_range(&self) -> u8 {
        self.inner.indicator_of_unit_of_time_range.unwrap()
    }

    /// 予報時間を返す。
    ///
    /// # 戻り値
    ///
    /// 予報時間
    pub fn forecast_time(&self) -> i32 {
        self.inner.forecast_time.unwrap()
    }

    /// 第一固定面の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 第一固定面の種類
    pub fn first_fixed_surface_type(&self) -> u8 {
        self.inner.type_of_first_fixed_surface.unwrap()
    }

    /// 全時間間隔の終了時を返す。
    ///
    /// # 戻り値
    ///
    /// 全時間間隔の終了時
    pub fn end_of_all_time_intervals(&self) -> PrimitiveDateTime {
        self.inner.end_of_all_time_intervals.unwrap()
    }

    /// 統計を算出するために使用した時間間隔を記述する期間の仕様の数を返す。
    ///
    /// # 戻り値
    ///
    /// 統計を算出するために使用した時間間隔を記述する期間の仕様の数
    pub fn number_of_time_range_specs(&self) -> u8 {
        self.inner.number_of_time_range_specs.unwrap()
    }

    /// 統計処理における欠測資料の総数を返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理における欠測資料の総数
    pub fn number_of_missing_values(&self) -> u32 {
        self.inner.number_of_missing_values.unwrap()
    }

    /// 統計処理の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理の種類
    pub fn type_of_stat_proc(&self) -> u8 {
        self.inner.type_of_stat_proc.unwrap()
    }

    /// 統計処理の時間増分の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理の時間増分の種類
    pub fn type_of_stat_proc_time_increment(&self) -> u8 {
        self.inner.type_of_stat_proc_time_increment.unwrap()
    }

    /// 統計処理の時間の単位の指示符を返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理の時間の単位の指示符
    pub fn stat_proc_time_unit(&self) -> u8 {
        self.inner.stat_proc_time_unit.unwrap()
    }

    /// 統計処理した時間の長さを返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理した時間の長さ
    pub fn stat_proc_time_length(&self) -> u32 {
        self.inner.stat_proc_time_length.unwrap()
    }

    /// 連続的な資料場間の増分に関する時間の単位の指示符を返す。
    ///
    /// # 戻り値
    ///
    /// 連続的な資料場間の増分に関する時間の単位の指示符
    pub fn successive_time_unit(&self) -> u8 {
        self.inner.successive_time_unit.unwrap()
    }

    /// 連続的な資料場間の時間の増分を返す。
    ///
    /// # 戻り値
    ///
    /// 連続的な資料場間の時間の増分
    pub fn successive_time_increment(&self) -> u32 {
        self.inner.successive_time_increment.unwrap()
    }

    /// レーダー等運用情報その1を返す。
    ///
    /// # 戻り値
    ///
    /// レーダー等運用情報その1
    pub fn radar_info1(&self) -> u64 {
        self.inner.radar_info1.unwrap()
    }

    /// レーダー等運用情報その2を返す。
    ///
    /// # 戻り値
    ///
    /// レーダー等運用情報その2
    pub fn radar_info2(&self) -> u64 {
        self.inner.radar_info2.unwrap()
    }

    /// 雨量計運用情報を返す。
    ///
    /// # 戻り値
    ///
    /// 雨量計運用情報
    pub fn rain_gauge_info(&self) -> u64 {
        self.inner.rain_gauge_info.unwrap()
    }

    /// 第5節に記録されている全資料点の数を返す。
    ///
    /// # 戻り値
    ///
    /// 第5節に記録されている全資料点の数
    pub fn number_of_values(&self) -> u32 {
        self.inner.number_of_values.unwrap()
    }

    /// 資料表現テンプレート番号を返す。
    ///
    /// # 戻り値
    ///
    /// 資料表現テンプレート番号
    pub fn data_representation_template_number(&self) -> u16 {
        self.inner.data_representation_template_number.unwrap()
    }

    /// 1データのビット数を返す。
    ///
    /// # 戻り値
    ///
    /// 1データのビット数
    pub fn bits_per_value(&self) -> u8 {
        self.inner.bits_per_value.unwrap()
    }

    /// 今回の圧縮に用いたレベルの最大値を返す。
    ///
    /// # 戻り値
    ///
    /// 圧縮に用いたレベルの最大値
    pub fn max_level_value(&self) -> u16 {
        self.inner.max_level_value.unwrap()
    }

    /// データの取り得るレベルの最大値を返す。
    ///
    /// # 戻り値
    ///
    /// データの取り得るレベルの最大値
    pub fn number_of_level_values(&self) -> u16 {
        self.inner.number_of_level_values.unwrap()
    }

    /// データ代表値の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// データ代表値の尺度因子
    pub fn decimal_scale_factor(&self) -> u8 {
        self.inner.decimal_scale_factor.unwrap()
    }

    /// レベルmに対応するデータ代表値を返す。
    ///
    /// # 戻り値
    ///
    /// レベル値と物理値(mm/h)の対応を格納するコレクション
    pub fn level_values(&self) -> &[u16] {
        self.inner.level_values.as_ref().unwrap()
    }

    /// ビットマップ指示符を返す。
    ///
    /// # 戻り値
    ///
    /// ビットマップ指示符
    pub fn bitmap_indicator(&self) -> u8 {
        self.inner.bitmap_indicator.unwrap()
    }

    /// ランレングス圧縮符号列の開始位置を返す。
    ///
    /// # 戻り値
    ///
    /// ランレングス圧縮符号列の開始位置
    pub fn run_length_position(&self) -> usize {
        self.inner.run_length_position.unwrap()
    }

    /// ランレングス圧縮符号列のバイト数を返す。
    ///
    /// # 戻り値
    ///
    /// ランレングス圧縮符号列のバイト数
    pub fn run_length_bytes(&self) -> usize {
        self.inner.run_length_bytes.unwrap()
    }

    /// ランレングス圧縮符号を走査するイテレーターを返す。
    ///
    /// # 戻り値
    ///
    /// ランレングス圧縮符号を走査するイテレーター
    pub fn values(&mut self) -> ReaderResult<Grib2ValueIter<'_>> {
        let file = File::open(self.path.as_ref())
            .map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = BufReader::new(file);
        reader
            .seek(SeekFrom::Start(self.run_length_position() as u64))
            .map_err(|_| {
                ReaderError::ReadError("ランレングス圧縮符号列のシークに失敗しました。".into())
            })?;

        Ok(Grib2ValueIter::new(
            reader,
            self.run_length_bytes(),
            self.number_of_data_points(),
            self.lat_of_first_grid_point(),
            self.lon_of_first_grid_point(),
            self.lon_of_last_grid_point(),
            self.j_direction_increment(),
            self.i_direction_increment(),
            self.bits_per_value() as u16,
            self.max_level_value(),
            self.level_values(),
        ))
    }

    /// 第0節:指示節を出力する。
    #[rustfmt::skip]
    pub fn write_section0<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第0節:指示節")?;
        writeln!(writer, "    資料分野: {}", self.discipline())?;
        writeln!(writer, "    GRIB版番号: {}", self.edition_number())?;
        writeln!(writer, "    GRIB報全体の長さ: 0x{:08X}", self.total_length())?;

        Ok(())
    }

    /// 第1節:識別節を出力する。
    #[rustfmt::skip]
    pub fn write_section1<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第1節:識別節")?;
        writeln!(writer, "    作成中枢の識別: {}", self.center())?;
        writeln!(writer, "    作成副中枢: {}", self.sub_center())?;
        writeln!(writer, "    GRIBマスター表バージョン番号: {}", self.table_version())?;
        writeln!(writer, "    GRIB地域表バージョン番号: {}", self.local_table_version())?;
        writeln!(writer, "    参照時刻の意味: {}", self.significance_of_reference_time())?;
        writeln!(writer, "    資料の参照時刻: {}", self.referenced_at())?;
        writeln!(writer, "    作成ステータス: {}", self.production_status_of_processed_data())?;
        writeln!(writer, "    資料の種類: {}", self.type_of_processed_data())?;

        Ok(())
    }

    /// 第2節:地域使用節を出力する。
    #[rustfmt::skip]
    pub fn write_section2<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第2節:地域使用節")?;

        Ok(())
    }

    /// 第3節:格子系定義節を出力する。
    #[rustfmt::skip]
    pub fn write_section3<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第3節:格子系定義節")?;
        writeln!(writer, "    格子系定義の出典: {}", self.source_of_grid_definition())?;
        writeln!(writer, "    資料点数: {}", self.number_of_data_points())?;
        writeln!(writer, "    格子点数を定義するリストのオクテット数: {}", self.number_of_octets_for_number_of_points())?;
        writeln!(writer, "    格子点数を定義するリストの説明: {}", self.interpretation_of_number_of_points())?;
        writeln!(writer, "    格子系定義テンプレート番号: {}", self.grid_definition_template_number())?;
        writeln!(writer, "    地球の形状: {}", self.shape_of_earth())?;
        writeln!(writer, "    地球回転楕円体の長軸の尺度因子: {}", self.scale_factor_of_earth_major_axis())?;
        writeln!(writer, "    地球回転楕円体の長軸の尺度付きの長さ: {}", self.scaled_value_of_earth_major_axis())?;
        writeln!(writer, "    地球回転楕円体の短軸の尺度因子: {}", self.scale_factor_of_earth_minor_axis())?;
        writeln!(writer, "    地球回転楕円体の短軸の尺度付きの長さ: {}", self.scaled_value_of_earth_minor_axis())?;
        writeln!(writer, "    緯線に沿った格子点数: {}", self.number_of_along_lat_points())?;
        writeln!(writer, "    経線に沿った格子点数: {}", self.number_of_along_lon_points())?;
        writeln!(writer, "    原作成領域の基本角: {}", self.basic_angle_of_initial_product_domain())?;
        writeln!(writer, "    最初の格子点の緯度: {}", self.lat_of_first_grid_point())?;
        writeln!(writer, "    最初の格子点の経度: {}", self.lon_of_first_grid_point())?;
        writeln!(writer, "    分解能及び成分フラグ: 0x{:02X}", self.resolution_and_component_flags())?;
        writeln!(writer, "    最後の格子点の緯度: {}", self.lat_of_last_grid_point())?;
        writeln!(writer, "    最後の格子点の経度: {}", self.lon_of_last_grid_point())?;
        writeln!(writer, "    i方向の増分: {}", self.j_direction_increment())?;
        writeln!(writer, "    j方向の増分: {}", self.i_direction_increment())?;
        writeln!(writer, "    走査モード: 0x{:02X}", self.scanning_mode())?;

        Ok(())
    }

    /// 第4節:プロダクト定義節を出力する。
    #[rustfmt::skip]
    pub fn write_section4<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第4節:プロダクト定義節")?;
        writeln!(writer, "    資料点数: {}", self.number_of_data_points())?;
        writeln!(writer, "    テンプレート直後の座標値の数: {}", self.number_of_after_template_points())?;
        writeln!(writer, "    プロダクト定義テンプレート番号: {}", self.product_definition_template_number())?;
        writeln!(writer, "    パラメータカテゴリー: {}", self.parameter_category())?;
        writeln!(writer, "    パラメータ番号: {}", self.parameter_number())?;
        writeln!(writer, "    作成処理の種類: {}", self.type_of_generating_process())?;
        writeln!(writer, "    背景作成処理識別符: {}", self.background_process())?;
        writeln!(writer, "    観測資料の参照時刻からの締切時間(時): {}", self.hours_after_data_cutoff())?;
        writeln!(writer, "    観測資料の参照時刻からの締切時間(分): {}", self.minutes_after_data_cutoff())?;
        writeln!(writer, "    期間の単位の指示符: {}", self.indicator_of_unit_of_time_range())?;
        writeln!(writer, "    予報時間(分): {}", self.forecast_time())?;
        writeln!(writer, "    第一固定面の種類: {}", self.first_fixed_surface_type())?;
        writeln!(writer, "    全時間間隔の終了時: {}", self.end_of_all_time_intervals())?;
        writeln!(writer, "    統計を算出するために使用した時間間隔を記述する期間の仕様の数: {}", self.number_of_time_range_specs())?;
        writeln!(writer, "    統計処理における欠測資料の総数: {}", self.number_of_missing_values())?;
        writeln!(writer, "    統計処理の種類: {}", self.type_of_stat_proc())?;
        writeln!(writer, "    統計処理の時間増分の種類: {}", self.type_of_stat_proc_time_increment())?;
        writeln!(writer, "    統計処理の時間の単位の指示符: {}", self.stat_proc_time_unit())?;
        writeln!(writer, "    統計処理した期間の長さ: {}", self.stat_proc_time_length())?;
        writeln!(writer, "    連続的な資料場間の増分に関する時間の単位の指示符: {}", self.successive_time_unit())?;
        writeln!(writer, "    続的な資料場間の時間の増分: {}", self.successive_time_increment())?;
        writeln!(writer, "    レーダー等運用情報その1: 0x{:02X}", self.radar_info1())?;
        writeln!(writer, "    レーダー等運用情報その2: 0x{:02X}", self.radar_info2())?;
        writeln!(writer, "    雨量計運用情報: 0x{:02X}", self.rain_gauge_info())?;

        Ok(())
    }

    /// 第5節:資料表現節を出力する。
    #[rustfmt::skip]
    pub fn write_section5<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第5節:資料表現節")?;
        writeln!(writer, "    全資料点の数: {}", self.number_of_values())?;
        writeln!(writer, "    資料表現テンプレート番号: {}", self.data_representation_template_number())?;
        writeln!(writer, "    1データのビット数: {}", self.bits_per_value())?;
        writeln!(writer, "    今回の圧縮に用いたレベルの最大値: {}", self.max_level_value())?;
        writeln!(writer, "    データの取り得るレベルの最大値: {}", self.number_of_level_values())?;
        writeln!(writer, "    データ代表値の尺度因子: {}", self.decimal_scale_factor())?;
        writeln!(writer, "    レベルmに対応するデータ代表値:")?;
        for (i, level_value) in self.level_values().iter().enumerate() {
            writeln!(writer, "        レベル{}: {}", i + 1, level_value)?;
        }

        Ok(())
    }

    /// 第6節:ビットマップ節を出力する。
    #[rustfmt::skip]
    pub fn write_section6<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第6節:ビットマップ節")?;
        writeln!(writer, "    ビットマップ指示符数: {}", self.bitmap_indicator())?;

        Ok(())
    }

    /// 第7節:資料節を出力する。
    #[rustfmt::skip]
    pub fn write_section7<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第7節:資料節")?;
        writeln!(writer, "    ランレングス圧縮符号開始位置: 0x{:08X}", self.run_length_position())?;
        writeln!(writer, "    ランレングス圧縮符号長さ: 0x{:08X}", self.run_length_bytes())?;

        Ok(())
    }

    /// 全ての節を出力する。
    pub fn write_sections<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.write_section0(writer)?;
        writeln!(writer)?;
        self.write_section1(writer)?;
        writeln!(writer)?;
        self.write_section2(writer)?;
        writeln!(writer)?;
        self.write_section3(writer)?;
        writeln!(writer)?;
        self.write_section4(writer)?;
        writeln!(writer)?;
        self.write_section5(writer)?;
        writeln!(writer)?;
        self.write_section6(writer)?;
        writeln!(writer)?;
        self.write_section7(writer)?;
        writeln!(writer)?;

        Ok(())
    }
}

/// Grib2Readerの内部構造体
#[derive(Debug, Default)]
struct Inner {
    /// 読み込んだバイト数
    read_bytes: usize,

    /// 第0節:指示節
    /// 資料分野
    discipline: Option<u8>,
    /// GRIB版番号
    edition_number: Option<u8>,
    /// GRIB報全体のバイト数
    total_length: Option<usize>,

    /// 第2節:識別節
    /// 作成中枢の識別
    center: Option<u16>,
    /// 作成副中枢
    sub_center: Option<u16>,
    /// GRIBマスター表バージョン番号
    table_version: Option<u8>,
    /// GRIB地域表バージョン番号
    local_table_version: Option<u8>,
    /// 参照時刻の意味
    significance_of_reference_time: Option<u8>,
    /// 資料の参照時刻
    referenced_at: Option<PrimitiveDateTime>,
    /// 作成ステータス
    production_status_of_processed_data: Option<u8>,
    /// 資料の種類
    type_of_processed_data: Option<u8>,

    /// 第3節:格子系定義節
    /// 格子系定義の出典
    source_of_grid_definition: Option<u8>,
    /// 第3節に記録されている資料点数
    number_of_data_points: Option<u32>,
    /// 格子点数を定義するリストのオクテット数
    number_of_octets_for_number_of_points: Option<u8>,
    /// 格子点数を定義するリストの説明
    interpretation_of_number_of_points: Option<u8>,
    /// 格子系定義テンプレート番号
    grid_definition_template_number: Option<u16>,
    /// 地球の形状
    shape_of_earth: Option<u8>,
    /// 地球回転楕円体の長軸の尺度因子
    scale_factor_of_earth_major_axis: Option<u8>,
    /// 地球回転楕円体の長軸の尺度付きの長さ
    scaled_value_of_earth_major_axis: Option<u32>,
    /// 地球回転楕円体の短軸の尺度因子
    scale_factor_of_earth_minor_axis: Option<u8>,
    /// 地球回転楕円体の短軸の尺度付きの長さ
    scaled_value_of_earth_minor_axis: Option<u32>,
    /// 緯線に沿った格子点数
    number_of_along_lat_points: Option<u32>,
    /// 経線に沿った格子点数
    number_of_along_lon_points: Option<u32>,
    /// 原作成領域の基本角
    basic_angle_of_initial_product_domain: Option<u32>,
    /// 最初の格子点の緯度（10e-6度単位）
    lat_of_first_grid_point: Option<u32>,
    /// 最初の格子点の経度（10e-6度単位）
    lon_of_first_grid_point: Option<u32>,
    /// 分解能及び成分フラグ
    resolution_and_component_flags: Option<u8>,
    /// 最後の格子点の緯度（10e-6度単位）
    lat_of_last_grid_point: Option<u32>,
    /// 最後の格子点の経度（10e-6度単位）
    lon_of_last_grid_point: Option<u32>,
    /// i方向（経度方向）の増分（10e-6度単位）
    i_direction_increment: Option<u32>,
    /// j方向（緯度方向）の増分（10e-6度単位）
    j_direction_increment: Option<u32>,
    /// 走査モード
    scanning_mode: Option<u8>,

    /// 第4章:プロダクト定義節
    /// テンプレート直後の座標値の数
    number_of_after_template_points: Option<u16>,
    /// プロダクト定義テンプレート番号
    product_definition_template_number: Option<u16>,
    /// パラメータカテゴリー
    parameter_category: Option<u8>,
    /// パラメータ番号
    parameter_number: Option<u8>,
    /// 作成処理の種類
    type_of_generating_process: Option<u8>,
    /// 背景作成処理識別符
    background_process: Option<u8>,
    /// 観測資料の参照時刻からの締切時間（時）
    hours_after_data_cutoff: Option<u16>,
    /// 観測資料の参照時刻からの締切時間（分）
    minutes_after_data_cutoff: Option<u8>,
    /// 期間の単位の指示符
    indicator_of_unit_of_time_range: Option<u8>,
    /// 予報時間
    forecast_time: Option<i32>,
    /// 第一固定面の種類
    type_of_first_fixed_surface: Option<u8>,
    /// 全時間間隔の終了時
    end_of_all_time_intervals: Option<PrimitiveDateTime>,
    /// 統計を算出するために使用した時間間隔を記述する期間の仕様の数
    number_of_time_range_specs: Option<u8>,
    /// 統計処理における欠測資料の総数
    number_of_missing_values: Option<u32>,
    /// 統計処理の種類
    type_of_stat_proc: Option<u8>,
    /// 統計処理の時間増分の種類
    type_of_stat_proc_time_increment: Option<u8>,
    /// 統計処理の時間の単位の指示符
    stat_proc_time_unit: Option<u8>,
    /// 統計処理した時間の長さ
    stat_proc_time_length: Option<u32>,
    /// 連続的な資料場間の増分に関する時間の単位の指示符
    successive_time_unit: Option<u8>,
    /// 連続的な資料場間の時間の増分
    successive_time_increment: Option<u32>,
    /// レーダー等運用情報その1
    radar_info1: Option<u64>,
    /// レーダー等運用情報その2
    radar_info2: Option<u64>,
    /// 雨量計運用情報
    rain_gauge_info: Option<u64>,

    /// 第5節:資料表現節
    /// 第5節に記録されている全資料点の数
    number_of_values: Option<u32>,
    /// 資料表現テンプレート番号
    data_representation_template_number: Option<u16>,
    /// 1データのビット数
    bits_per_value: Option<u8>,
    /// 今回の圧縮に用いたレベルの最大値
    max_level_value: Option<u16>,
    /// データの取り得るレベルの最大値
    number_of_level_values: Option<u16>,
    /// データ代表値の尺度因子
    decimal_scale_factor: Option<u8>,
    /// レベルmに対応するデータ代表値
    /// レベル値と物理値(mm/h)の対応を格納するコレクション
    level_values: Option<Vec<u16>>,

    /// 第6節:ビットマップ節
    /// ビットマップ指示符
    bitmap_indicator: Option<u8>,

    /// 第7節:資料値節
    /// ランレングス圧縮符号列の開始位置
    run_length_position: Option<usize>,
    /// ランレングス圧縮符号列のバイト数
    run_length_bytes: Option<usize>,
}

/// `Inner`構造体が実装する数値を読み込むメソッドに展開するマクロ
macro_rules! read_number {
    ($fname:ident, $type:ty) => {
        fn $fname(&mut self, reader: &mut FileReader) -> ReaderResult<$type> {
            let expected_bytes = std::mem::size_of::<$type>();
            let mut buf = vec![0_u8; expected_bytes];
            reader.read_exact(&mut buf).map_err(|_| {
                ReaderError::ReadError(
                    format!("{}バイト読み込めませんでした。", expected_bytes).into(),
                )
            })?;
            self.read_bytes += expected_bytes;

            Ok(<$type>::from_be_bytes(buf.try_into().unwrap()))
        }
    };
}

/// `Inner`構造体が実装する読み込んだ数値を検証するメソッドに展開するマクロ
macro_rules! validate_number {
    ($fname:ident, $read_fn:ident, $type:ty, $name:ident, $fmt:ident) => {
        fn $fname(
            &mut self,
            reader: &mut FileReader,
            expected: $type,
            $name: &str,
            fmt: &str,
        ) -> ReaderResult<$type> {
            let value = self.$read_fn(reader).map_err(|_| {
                ReaderError::ReadError(format!("{}の読み込みに失敗しました。", $name).into())
            })?;
            if value != expected {
                let msg = fmt
                    .replace("{value}", &value.to_string())
                    .replace("{expected}", &expected.to_string());
                return Err(ReaderError::Unexpected(msg.into()));
            }

            Ok(value)
        }
    };
}

impl Inner {
    fn read_to_end(&mut self, mut reader: FileReader) -> ReaderResult<()> {
        // 第0節:指示節 読み込み
        self.read_section0(&mut reader)?;
        // 第1節:識別節 読み込み
        self.read_section1(&mut reader)?;
        // 第2節:地域使用節 読み込み
        self.read_section2(&mut reader)?;
        // 第3節:格子系定義節 読み込み
        self.read_section3(&mut reader)?;
        // 第4節:プロダクト定義節 読み込み
        self.read_section4(&mut reader)?;
        // 第5節:資料表現節 読み込み
        self.read_section5(&mut reader)?;
        // 第6節:ビットマップ節 読み込み
        self.read_section6(&mut reader)?;
        // 第7節:資料値節 読み込み
        self.read_section7(&mut reader)?;
        // 第8節:終端節 読み込み
        self.read_section8(&mut reader)?;

        Ok(())
    }

    /// 第0節:指示節を読み込み。
    ///
    /// # 引数
    ///
    /// * `reader` - ファイルリーダー
    ///
    /// # 戻り値
    ///
    /// `()`
    fn read_section0(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        // GRIB: 4バイト
        let grib = self.read_str(reader, 4).map_err(|e| {
            ReaderError::ReadError(format!("第0節:GRIBの読み込みに失敗しました。{}", e).into())
        })?;
        if grib != "GRIB" {
            return Err(ReaderError::ReadError(
                "第0節:GRIBを読み込めませんでした。".into(),
            ));
        }

        // 保留: 2バイト
        self.seek_relative(reader, 2).map_err(|_| {
            ReaderError::ReadError("第0節:保留(5-6オクテット)の読み飛ばしに失敗しました。".into())
        })?;

        // 資料分野: 1バイト
        self.discipline = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第0節:資料分野の読み込みに失敗しました。".into())
        })?);

        // GRIB版番号: 1バイト
        self.edition_number = Some(self.validate_u8(
            reader,
            EDITION_NUMBER,
            "GRIB版番号",
            "GRIB版番号の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // GRIB報全体の長さ: 8バイト
        self.total_length = Some(self.read_u64(reader).map_err(|_| {
            ReaderError::ReadError("第0節:GRIB報全体の長さの読み込みに失敗しました。".into())
        })? as usize);

        // 検証
        if SECTION0_BYTES != self.read_bytes {
            return Err(ReaderError::ReadError(
                format!(
                    "第0節で読み込んだサイズ({})と定義({})が異なります。",
                    self.read_bytes, SECTION0_BYTES
                )
                .into(),
            ));
        }

        Ok(())
    }

    /// 第1節:識別節を読み込む。
    ///
    /// ファイルポインタが、第1節の開始位置にあることを想定している。
    /// 関数終了後、ファイルポインタは第3節の開始位置に移動する。
    /// なお、実装時点で、第2節は省略されている。
    fn read_section1(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        let to_section0_bytes = self.read_bytes;

        // 節の長さ: 4bytes
        let section_bytes = self.validate_u32(
            reader,
            SECTION1_BYTES,
            "節の長さ",
            "節の長さの値は{value}でしたが、{expected}でなければなりません。",
        )? as usize;

        // 節番号
        self.validate_u8(
            reader,
            1,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 作成中枢の識別: 2bytes
        self.center = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError("第1節:作成中枢の識別の読み込みに失敗しました。".into())
        })?);

        // 作成副中枢: 2bytes
        self.sub_center = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError("第1節:作成副中枢の読み込みに失敗しました。".into())
        })?);

        // GRIBマスター表バージョン番号: 1byte
        self.table_version = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第1節:GRIBマスター表バージョン番号の読み込みに失敗しました。".into(),
            )
        })?);

        // GRIB地域表バージョン番号: 1byte
        self.local_table_version = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第1節:GRIB地域表バージョン番号の読み込みに失敗しました。".into(),
            )
        })?);

        // 参照時刻の意味: 1byte
        self.significance_of_reference_time = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第1節:参照時刻の意味の読み込みに失敗しました。".into())
        })?);

        // 資料の参照時刻（日時）
        self.referenced_at = Some(self.read_datetime(reader).map_err(|_| {
            ReaderError::ReadError("第1節:資料の参照時刻の読み込みに失敗しました。".into())
        })?);

        // 作成ステータス
        self.production_status_of_processed_data = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第1節:作成ステータスの読み込みに失敗しました。".into())
        })?);

        // 資料の種類
        self.type_of_processed_data = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第1節:資料の種類の読み込みに失敗しました。".into())
        })?);

        // 検証
        let section_read_bytes = self.read_bytes - to_section0_bytes;
        if section_bytes != section_read_bytes {
            return Err(ReaderError::ReadError(
                format!(
                    "第1節で読み込んだサイズ({})と定義({})が異なります。",
                    section_read_bytes, section_bytes
                )
                .into(),
            ));
        }

        Ok(())
    }

    /// 第2節:地域使用節を読み込む。
    fn read_section2(&mut self, _reader: &mut FileReader) -> ReaderResult<()> {
        Ok(())
    }

    /// 第3節:格子系定義節を読み込む。
    fn read_section3(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        let to_section2_bytes = self.read_bytes;

        // 節の長さ: 4バイト
        let section_bytes = self.validate_u32(
            reader,
            SECTION3_BYTES,
            "節の長さ",
            "節の長さの値は{value}でしたが、{expected}でなければなりません。",
        )? as usize;

        // 節番号: 1バイト
        self.validate_u8(
            reader,
            3,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 格子系定義の出典: 1バイト
        self.source_of_grid_definition = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第3節:格子系定義の出典の読み込みに失敗しました。".into())
        })?);

        // 資料点数: 4バイト
        self.number_of_data_points = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第3節:格子点数の読み込みに失敗しました。".into())
        })?);

        // 格子点数を定義するリストのオクテット数: 1バイト
        self.number_of_octets_for_number_of_points = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:格子点数を定義するリストのオクテット数の読み込みに失敗しました。".into(),
            )
        })?);

        // 格子点数を定義するリストの説明
        self.interpretation_of_number_of_points = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:格子点数を定義するリストの説明の読み込みに失敗しました。".into(),
            )
        })?);

        // 格子系定義テンプレート番号: 2バイト
        self.grid_definition_template_number = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:格子系定義テンプレート番号の読み込みに失敗しました。".into(),
            )
        })?);

        // 地球の形状: 1バイト
        self.shape_of_earth = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第3節:地球の形状の読み込みに失敗しました。".into())
        })?);

        // 地球球体の半径の尺度因子: 1バイト
        self.seek_relative(reader, 1).map_err(|_| {
            ReaderError::ReadError(
                "第3節:地球球体の半径の尺度因子の読み飛ばしに失敗しました。".into(),
            )
        })?;

        // 地球球体の尺度付き半径: 4バイト
        self.seek_relative(reader, 4).map_err(|_| {
            ReaderError::ReadError(
                "第3節:地球球体の尺度付き半径の読み飛ばしに失敗しました。".into(),
            )
        })?;

        // 地球回転楕円体の長軸の尺度因子: 1バイト
        self.scale_factor_of_earth_major_axis = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:地球回転楕円体の長軸の尺度因子の読み込みに失敗しました。".into(),
            )
        })?);

        // 地球回転楕円体の長軸の尺度付きの長さ: 4バイト
        self.scaled_value_of_earth_major_axis = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:地球回転楕円体の長軸の尺度付きの長さの読み込みに失敗しました。".into(),
            )
        })?);

        // 地球回転楕円体の短軸の尺度因子: 1バイト
        self.scale_factor_of_earth_minor_axis = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:地球回転楕円体の短軸の尺度因子の読み込みに失敗しました。".into(),
            )
        })?);

        // 地球回転楕円体の短軸の尺度付きの長さ: 4バイト
        self.scaled_value_of_earth_minor_axis = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:地球回転楕円体の短軸の尺度付きの長さの読み込みに失敗しました。".into(),
            )
        })?);

        // 緯線に沿った格子点数: 4バイト
        self.number_of_along_lat_points = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第3節:緯線に沿った格子点数の読み込みに失敗しました。".into())
        })?);

        // 経線に沿った格子点数: 4バイト
        self.number_of_along_lon_points = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第3節:経線に沿った格子点数の読み込みに失敗しました。".into())
        })?);

        // 原作成領域の基本角: 4バイト
        self.basic_angle_of_initial_product_domain = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第3節:原作成領域の基本角の読み込みに失敗しました。".into())
        })?);

        // 端点の経度及び緯度並びに方向増分の定義に使われる基本角の細分: 4バイト
        self.seek_relative(reader, 4).map_err(|_| {
            ReaderError::ReadError(
                "第3節:端点の経度及び緯度並びに方向増分の定義に使われる基本角の細分の読み飛ばしに失敗しました。"
                    .into(),
            )
        })?;

        // 最初の格子点の緯度（10e-6度単位）: 4バイト
        self.lat_of_first_grid_point = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:最初の格子点の緯度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 最初の格子点の経度（10e-6度単位）: 4バイト
        self.lon_of_first_grid_point = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:最初の格子点の経度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 分解能及び成分フラグ: 1バイト
        self.resolution_and_component_flags = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第3節:分解能及び成分フラグの読み込みに失敗しました。".into())
        })?);

        // 最後の格子点の緯度（10e-6度単位）: 4バイト
        self.lat_of_last_grid_point = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:最後の格子点の緯度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 最後の格子点の経度（10e-6度単位）: 4バイト
        self.lon_of_last_grid_point = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:最後の格子点の経度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // i方向（経度方向）の増分（10e-6度単位）: 4バイト
        self.i_direction_increment = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:i方向（経度方向）の増分（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // j方向（緯度方向）の増分（10e-6度単位）: 4バイト
        self.j_direction_increment = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:j方向（緯度方向）の増分（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 走査モード: 1バイト
        self.scanning_mode = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第3節:走査モードの読み込みに失敗しました。".into())
        })?);

        let section_read_bytes = self.read_bytes - to_section2_bytes;
        if section_bytes != section_read_bytes {
            return Err(ReaderError::ReadError(
                format!(
                    "第3節で読み込んだサイズ({})と定義({})が異なります。",
                    section_read_bytes, section_bytes
                )
                .into(),
            ));
        }

        Ok(())
    }

    /// 第4節:プロダクト定義節を読み込む。
    fn read_section4(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        // 第3節までの読み込んだバイト数を記憶
        let to_section3_bytes = self.read_bytes;

        // 節の長さ: 4バイト
        let section_bytes = self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第4節:節の長さの読み込みに失敗しました。".into())
        })? as usize;

        // 節番号: 1バイト
        self.validate_u8(
            reader,
            4,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // テンプレート直後の座標値の数: 2バイト
        self.number_of_after_template_points = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:テンプレート直後の座標値の数の読み込みに失敗しました。".into(),
            )
        })?);

        // プロダクト定義テンプレート番号: 2バイト
        self.product_definition_template_number = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:プロダクト定義テンプレート番号の読み込みに失敗しました。".into(),
            )
        })?);

        // パラメータカテゴリー: 1バイト
        self.parameter_category = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:パラメータカテゴリーの読み込みに失敗しました。".into())
        })?);

        // パラメータ番号: 1バイト
        self.parameter_number = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:パラメータ番号の読み込みに失敗しました。".into())
        })?);

        // 作成処理の種類: 1バイト
        self.type_of_generating_process = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:作成処理の種類の読み込みに失敗しました。".into())
        })?);

        // 背景作成処理識別符: 1バイト
        self.background_process = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:背景作成処理識別符の読み込みに失敗しました。".into())
        })?);

        // 予報の作成処理識別符: 1バイト
        self.seek_relative(reader, 1).map_err(|_| {
            ReaderError::ReadError("第4節:予報の作成処理識別符の読み飛ばしに失敗しました。".into())
        })?;

        // 観測資料の参照時刻からの締切時間（時）: 2バイト
        self.hours_after_data_cutoff = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:観測資料の参照時刻からの締切時間（時）の読み込みに失敗しました。".into(),
            )
        })?);

        // 観測資料の参照時刻からの締切時間（分）: 1バイト
        self.minutes_after_data_cutoff = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:観測資料の参照時刻からの締切時間（分）の読み込みに失敗しました。".into(),
            )
        })?);

        // 期間の単位の指示符: 1バイト
        self.indicator_of_unit_of_time_range = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:期間の単位の指示符の読み込みに失敗しました。".into())
        })?);

        // 予報時間: 4バイト
        self.forecast_time = Some(self.read_i32(reader).map_err(|_| {
            ReaderError::ReadError("第4節:予報時間の読み込みに失敗しました。".into())
        })?);

        // 第一固定面の種類: 1バイト
        self.type_of_first_fixed_surface = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:第一固定面の種類の読み込みに失敗しました。".into())
        })?);

        // 第一固定面の尺度因子: 1バイト
        self.seek_relative(reader, 1).map_err(|_| {
            ReaderError::ReadError("第4節:第一固定面の尺度因子の読み飛ばしに失敗しました。".into())
        })?;

        // 第一固定面の尺度付きの値: 4バイト
        self.seek_relative(reader, 4).map_err(|_| {
            ReaderError::ReadError(
                "第4節:第一固定面の尺度付きの値の読み飛ばしに失敗しました。".into(),
            )
        })?;

        // 第二固定面の種類: 1バイト
        self.seek_relative(reader, 1).map_err(|_| {
            ReaderError::ReadError("第4節:第二固定面の種類の読み飛ばしに失敗しました。".into())
        })?;

        // 第二固定面の尺度因子: 1バイト
        self.seek_relative(reader, 1).map_err(|_| {
            ReaderError::ReadError("第4節:第二固定面の尺度因子の読み飛ばしに失敗しました。".into())
        })?;

        // 第二固定面の尺度付きの値: 4バイト
        self.seek_relative(reader, 4).map_err(|_| {
            ReaderError::ReadError(
                "第4節:第二固定面の尺度付きの値の読み飛ばしに失敗しました。".into(),
            )
        })?;

        // 全時間間隔の終了時: 7バイト
        self.end_of_all_time_intervals = Some(self.read_datetime(reader).map_err(|_| {
            ReaderError::ReadError("第4節:全時間間隔の終了時の読み込みに失敗しました。".into())
        })?);

        // 統計を算出するために使用した時間間隔を記述する期間の仕様の数: 1バイト
        self.number_of_time_range_specs = Some(self.read_u8( reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:統計を算出するために使用した時間間隔を記述する期間の仕様の数の読み込みに失敗しました。".into(),
            )
        })?);

        // 統計処理における欠測資料の総数: 4バイト
        self.number_of_missing_values = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:統計処理における欠測資料の総数の読み込みに失敗しました。".into(),
            )
        })?);

        // 統計処理の種類: 1バイト
        self.type_of_stat_proc = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:統計処理の種類の読み込みに失敗しました。".into())
        })?);

        // 統計処理の時間増分の種類: 1バイト
        self.type_of_stat_proc_time_increment = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:統計処理の時間増分の種類の読み込みに失敗しました。".into(),
            )
        })?);

        // 統計処理の時間の単位の指示符: 1バイト
        self.stat_proc_time_unit = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:統計処理の時間の単位の指示符の読み込みに失敗しました。".into(),
            )
        })?);

        // 統計処理した期間の長さ: 4バイト
        self.stat_proc_time_length = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:統計処理の時間増分の長さの読み込みに失敗しました。".into(),
            )
        })?);

        // 連続的な資料場間の増分に関する時間の単位の指示符: 1バイト
        self.successive_time_unit = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:連続的な資料場間の増分に関する時間の単位の指示符の読み込みに失敗しました。"
                    .into(),
            )
        })?);

        // 連続的な資料場間の時間の増分: 4バイト
        self.successive_time_increment = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:連続的な資料場間の時間の増分の読み込みに失敗しました。".into(),
            )
        })?);

        // レーダー等運用情報その1: 8バイト
        self.radar_info1 = Some(self.read_u64(reader).map_err(|_| {
            ReaderError::ReadError("第4節:レーダー等運用情報その1の読み込みに失敗しました。".into())
        })?);

        // レーダー等運用情報その2: 8バイト
        self.radar_info2 = Some(self.read_u64(reader).map_err(|_| {
            ReaderError::ReadError("第4節:レーダー等運用情報その2の読み込みに失敗しました。".into())
        })?);

        // 雨量計運用情報: 8バイト
        self.rain_gauge_info = Some(self.read_u64(reader).map_err(|_| {
            ReaderError::ReadError("第4節:雨量計運用情報の読み込みに失敗しました。".into())
        })?);

        // 検証
        let section_read_bytes = self.read_bytes - to_section3_bytes;
        if section_bytes != section_read_bytes {
            return Err(ReaderError::ReadError(
                format!(
                    "第4節で読み込んだサイズ({})と定義({})が異なります。",
                    section_read_bytes, section_bytes
                )
                .into(),
            ));
        }

        Ok(())
    }

    /// 第5節:資料表現節を読み込み。
    fn read_section5(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        // 第4節までの読み込んだバイト数を記憶
        let to_section4_bytes = self.read_bytes;

        // 節の長さ: 4バイト
        let section_bytes = self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第5節:節の長さの読み込みに失敗しました。".into())
        })? as usize;

        // 節番号: 1バイト
        self.validate_u8(
            reader,
            5,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 全資料点の数: 4バイト
        self.number_of_values = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第5節:全資料点の数の読み込みに失敗しました。".into())
        })?);

        // 資料表現テンプレート番号: 2バイト
        self.data_representation_template_number = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError(
                "第5節:資料表現テンプレート番号の読み込みに失敗しました。".into(),
            )
        })?);

        // 1データのビット数: 1バイト
        self.bits_per_value = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第5節:1データのビット数の読み込みに失敗しました。".into())
        })?);

        // 今回の圧縮に用いたレベルの最大値: 2バイト
        self.max_level_value = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError(
                "第5節:今回の圧縮に用いたレベルの最大値の読み込みに失敗しました。".into(),
            )
        })?);

        // データの取り得るレベルの最大値: 2バイト
        self.number_of_level_values = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError("第5節:レベルの最大値の読み込みに失敗しました。".into())
        })?);

        // データ代表値の尺度因子: 1バイト
        self.decimal_scale_factor = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第5節:データ代表値の尺度因子の読み込みに失敗しました。".into())
        })?);

        // レベルmに対応するデータ代表値
        let remaining_bytes = (section_bytes - (4 + 1 + 4 + 2 + 1 + 2 + 2 + 1)) as u16;
        let number_of_levels = remaining_bytes / 2;
        let mut level_values = Vec::new();
        for _ in 0..number_of_levels {
            level_values.push(self.read_u16(reader).map_err(|_| {
                ReaderError::ReadError(
                    "第5節:レベルmに対応するデータ代表値の読み込みに失敗しました。".into(),
                )
            })?);
        }
        self.level_values = Some(level_values);

        // 検証
        let section_read_bytes = self.read_bytes - to_section4_bytes;
        if section_bytes != section_read_bytes {
            return Err(ReaderError::ReadError(
                format!(
                    "第5節で読み込んだサイズ({})と定義({})が異なります。",
                    section_read_bytes, section_bytes
                )
                .into(),
            ));
        }

        Ok(())
    }

    /// 第6節:ビットマップ節を読み込む。
    fn read_section6(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        // 第5節までの読み込んだバイト数を記憶
        let to_section5_bytes = self.read_bytes;

        // 節の長さ: 4バイト
        let section_bytes = self.validate_u32(
            reader,
            SECTION6_BYTES,
            "節の長さ",
            "節の長さの値は{value}でしたが、{expected}でなければなりません。",
        )? as usize;

        // 節番号: 1バイト
        self.validate_u8(
            reader,
            6,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // ビットマップ指示符: 1バイト
        self.bitmap_indicator = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第6節:ビットマップ指示符の読み込みに失敗しました。".into())
        })?);

        // 検証
        let section_read_bytes = self.read_bytes - to_section5_bytes;
        if section_bytes != section_read_bytes {
            return Err(ReaderError::ReadError(
                format!(
                    "第6節で読み込んだサイズ({})と定義({})が異なります。",
                    section_read_bytes, section_bytes
                )
                .into(),
            ));
        }

        Ok(())
    }

    /// 第7節:資料節を読み込む。
    fn read_section7(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        // 節の長さ: 4バイト
        let section7_bytes = self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第7節:節の長さの読み込みに失敗しました。".into())
        })? as usize;

        // 節番号: 1バイト
        self.validate_u8(
            reader,
            7,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // ランレングス圧縮符号列の開始位置を記憶
        self.run_length_position = Some(self.read_bytes);

        // ランレングス圧縮符号列をスキップ
        self.run_length_bytes = Some(section7_bytes - (4 + 1));
        self.seek_relative(reader, self.run_length_bytes.unwrap() as i64)
            .map_err(|_| {
                ReaderError::ReadError(
                    "第7節:ランレングス圧縮オクテット列の読み飛ばしに失敗しました。".into(),
                )
            })?;

        Ok(())
    }

    /// 第8節:終端節を読み込む。
    fn read_section8(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        let s = self.read_str(reader, 4).map_err(|e| {
            ReaderError::ReadError(format!("第8節の読み込みに失敗しました。{:?}", e).into())
        })?;
        if s != SECTION8_END_MARKER {
            return Err(ReaderError::ReadError(
                format!(
                    "第8節の終了が不正です。ファイルを正確に読み込めなかった可能性があります。expected: {}, actual: {}",
                    SECTION8_END_MARKER, s
                )
                .into(),
            ));
        }

        Ok(())
    }

    fn read_str(&mut self, reader: &mut FileReader, size: usize) -> ReaderResult<String> {
        let mut buf = vec![0; size];
        let read_size = reader.read(&mut buf).map_err(|_| {
            ReaderError::ReadError(
                format!("{}バイトの文字列の読み込みに失敗しました。", size).into(),
            )
        })?;
        if read_size != size {
            return Err(ReaderError::ReadError(
                format!("{}バイトの文字列の読み込みに失敗しました。", size).into(),
            ));
        }
        self.read_bytes += size;

        Ok(String::from_utf8(buf).map_err(|_| {
            ReaderError::Unexpected(
                format!("{}バイトの文字列のコードに失敗しました。", size).into(),
            )
        }))?
    }

    fn read_datetime(&mut self, reader: &mut FileReader) -> ReaderResult<PrimitiveDateTime> {
        let year = self.read_u16(reader)?;
        let mut parts = Vec::new();
        for _ in 0..5 {
            parts.push(self.read_u8(reader)?);
        }
        // 日付と時刻を構築
        let month = Month::try_from(parts[0]).map_err(|_| {
            ReaderError::Unexpected(
                format!(
                    "月の値は{}でしたが、1から12の範囲でなければなりません。",
                    parts[0]
                )
                .into(),
            )
        })?;
        let date = Date::from_calendar_date(year as i32, month, parts[1]).map_err(|_| {
            ReaderError::Unexpected(
                format!(
                    "{}年{}月は{}日を日付に変換できませんでした。",
                    year, month as u8, parts[1]
                )
                .into(),
            )
        })?;
        let time = Time::from_hms(parts[2], parts[3], parts[4]).map_err(|_| {
            ReaderError::Unexpected(
                format!(
                    "{}時{}分{}秒を時刻に変換できませんでした。",
                    parts[2], parts[3], parts[4]
                )
                .into(),
            )
        })?;

        Ok(PrimitiveDateTime::new(date, time))
    }

    read_number!(read_u8, u8);
    read_number!(read_u16, u16);
    read_number!(read_u32, u32);
    read_number!(read_u64, u64);

    // read_number!(read_i8, i8);
    // read_number!(read_i16, i16);
    read_number!(read_i32, i32);
    // read_number!(read_i64, i64);

    validate_number!(validate_u8, read_u8, u8, name, fmt);
    // validate_number!(validate_u16, read_u16, u16, name, fmt);
    validate_number!(validate_u32, read_u32, u32, name, fmt);
    // validate_uint!(validate_u64, read_u64, u64, name, fmt);

    /// ファイルを指定されたバイト数読み飛ばす。
    ///
    /// # 引数
    ///
    /// * `offset` - 読み飛ばすバイト数
    ///
    /// # 戻り値
    ///
    /// `()`
    fn seek_relative(&mut self, reader: &mut FileReader, offset: i64) -> std::io::Result<()> {
        reader.seek_relative(offset)?;
        self.read_bytes += offset as usize;

        Ok(())
    }
}

/// 第0節
/// 節の長さ
const SECTION0_BYTES: usize = 16;
/// GRIB版番号
const EDITION_NUMBER: u8 = 2;

/// 第1節
/// 節の長さ（バイト）
const SECTION1_BYTES: u32 = 21;

/// 第3節
/// 節の長さ（バイト）
const SECTION3_BYTES: u32 = 72;

/// 第6節
/// 節の長さ（バイト）
const SECTION6_BYTES: u32 = 6;
/// ビットマップ指示符

/// 第8節
/// 第8節終端のマーカー
const SECTION8_END_MARKER: &str = "7777";
