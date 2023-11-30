use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::str;

use num_format::{Locale, ToFormattedString};
use time::{Date, Month, PrimitiveDateTime, Time};

use super::{FileReader, ReaderError, ReaderResult};
use crate::reader::grib2_value_iter::Grib2ValueIter;

/// Grib2Reader
pub struct Grib2Reader<P>
where
    P: AsRef<Path>,
{
    path: P,
    inner: Inner,
}

impl<P> Grib2Reader<P>
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
        if inner.number_of_points.unwrap() != inner.total_number_of_points.unwrap() {
            return Err(ReaderError::Unexpected(
                format!(
                    "第3節に記録されている資料点数({})と第5節に記録されている全資料点({})が一致しません。",
                    inner.number_of_points.unwrap().to_formatted_string(&Locale::ja),
                    inner.total_number_of_points.unwrap().to_formatted_string(&Locale::ja),
                ).into(),
            ));
        }

        Ok(Grib2Reader { path, inner })
    }

    /// 資料分野を返す。
    ///
    /// # 戻り値
    ///
    /// 資料分野
    pub fn document_domain(&self) -> u8 {
        self.inner.document_domain.unwrap()
    }

    /// GRIB版番号を返す。
    ///
    /// # 戻り値
    ///
    /// GRIB版番号
    pub fn grib_version(&self) -> u8 {
        self.inner.grib_version.unwrap()
    }

    /// GRIB報全体のバイト数を返す。
    ///
    ///
    /// # 戻り値
    ///
    /// GRIB報全体のバイト数
    pub fn total_bytes(&self) -> usize {
        self.inner.total_bytes.unwrap()
    }

    /// 作成中枢の識別を返す。
    ///
    /// # 戻り値
    ///
    /// 作成中枢の識別
    pub fn creator_identify(&self) -> u16 {
        self.inner.creator_identify.unwrap()
    }

    /// 作成副中枢を返す。
    ///
    /// # 戻り値
    ///
    /// 作成副中枢
    pub fn creator_sub_identify(&self) -> u16 {
        self.inner.creator_sub_identify.unwrap()
    }

    /// GRIBマスター表バージョン番号を返す。
    ///
    /// # 戻り値
    ///
    /// GRIBマスター表バージョン番号
    pub fn master_table_version(&self) -> u8 {
        self.inner.master_table_version.unwrap()
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
    pub fn reference_time_significance(&self) -> u8 {
        self.inner.reference_time_significance.unwrap()
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
    pub fn creation_status(&self) -> u8 {
        self.inner.creation_status.unwrap()
    }

    /// 資料の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 資料の種類
    pub fn document_kind(&self) -> u8 {
        self.inner.document_kind.unwrap()
    }

    /// 格子系定義の出典を返す。
    ///
    /// # 戻り値
    ///
    /// 格子系定義の出典
    pub fn frame_definition_source(&self) -> u8 {
        self.inner.frame_definition_source.unwrap()
    }

    /// 第3節に記録されている資料点数を返す。
    ///
    /// # 戻り値
    ///
    /// 第3節に記録されている資料点数
    pub fn number_of_points(&self) -> u32 {
        self.inner.number_of_points.unwrap()
    }

    /// 格子点数を定義するリストのオクテット数を返す。
    ///
    /// # 戻り値
    ///
    /// 格子点数を定義するリストのオクテット数
    pub fn frame_system_bytes(&self) -> u8 {
        self.inner.frame_system_bytes.unwrap()
    }

    /// 格子点数を定義するリストの説明を返す。
    ///
    /// # 戻り値
    ///
    /// 格子点数を定義するリストの説明
    pub fn frame_system_description(&self) -> u8 {
        self.inner.frame_system_description.unwrap()
    }

    /// 格子系定義テンプレート番号を返す。
    ///
    /// # 戻り値
    ///
    /// 格子系定義テンプレート番号
    pub fn frame_system_template(&self) -> u16 {
        self.inner.frame_system_template.unwrap()
    }

    /// 地球の形状を返す。
    ///
    /// # 戻り値
    ///
    /// 地球の形状
    pub fn earth_shape(&self) -> u8 {
        self.inner.earth_shape.unwrap()
    }

    /// 地球回転楕円体の長軸の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の長軸の尺度因子
    pub fn major_axis_scale_factor(&self) -> u8 {
        self.inner.major_axis_scale_factor.unwrap()
    }

    /// 地球回転楕円体の長軸の尺度付きの長さを返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の長軸の尺度付きの長さ
    pub fn major_axis_length(&self) -> u32 {
        self.inner.major_axis_length.unwrap()
    }

    /// 地球回転楕円体の短軸の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の短軸の尺度因子
    pub fn minor_axis_scale_factor(&self) -> u8 {
        self.inner.minor_axis_scale_factor.unwrap()
    }

    /// 地球回転楕円体の短軸の尺度付きの長さを返す。
    ///
    /// # 戻り値
    ///
    /// 地球回転楕円体の短軸の尺度付きの長さ
    pub fn minor_axis_length(&self) -> u32 {
        self.inner.minor_axis_length.unwrap()
    }

    /// 緯線に沿った格子点数を返す。
    ///
    /// # 戻り値
    ///
    /// 緯線に沿った格子点数
    pub fn number_of_points_lat(&self) -> u32 {
        self.inner.number_of_points_lat.unwrap()
    }

    /// 経線に沿った格子点数を返す。
    ///
    /// # 戻り値
    ///
    /// 経線に沿った格子点数
    pub fn number_of_points_lon(&self) -> u32 {
        self.inner.number_of_points_lon.unwrap()
    }

    /// 原作成領域の基本角を返す。
    ///
    /// # 戻り値
    ///
    /// 原作成領域の基本角
    pub fn basic_angle_area(&self) -> u32 {
        self.inner.basic_angle_area.unwrap()
    }

    /// 最初の格子点の緯度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最初の格子点の緯度（10e-6度単位）
    pub fn first_point_lat(&self) -> u32 {
        self.inner.first_point_lat.unwrap()
    }

    /// 最初の格子点の経度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最初の格子点の経度（10e-6度単位）
    pub fn first_point_lon(&self) -> u32 {
        self.inner.first_point_lon.unwrap()
    }

    /// 分解能及び成分フラグを返す。
    ///
    /// # 戻り値
    ///
    /// 分解能及び成分フラグ
    pub fn resolution_and_component(&self) -> u8 {
        self.inner.resolution_and_component.unwrap()
    }

    /// 最後の格子点の緯度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最後の格子点の緯度（10e-6度単位）
    pub fn last_point_lat(&self) -> u32 {
        self.inner.last_point_lat.unwrap()
    }

    /// 最後の格子点の経度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最後の格子点の経度（10e-6度単位）
    pub fn last_point_lon(&self) -> u32 {
        self.inner.last_point_lon.unwrap()
    }

    /// i方向（経度方向）の増分（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// i方向（経度方向）の増分（10e-6度単位）
    pub fn increment_lon(&self) -> u32 {
        self.inner.increment_lon.unwrap()
    }

    /// j方向（緯度方向）の増分（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// j方向（緯度方向）の増分（10e-6度単位）
    pub fn increment_lat(&self) -> u32 {
        self.inner.increment_lat.unwrap()
    }

    /// 走査モードを返す。
    ///
    /// # 戻り値
    ///
    /// 走査モード
    pub fn scan_mode(&self) -> u8 {
        self.inner.scan_mode.unwrap()
    }

    /// テンプレート直後の座標値の数を返す。
    ///
    /// # 戻り値
    ///
    /// テンプレート直後の座標値の数
    pub fn number_of_points_after_template(&self) -> u16 {
        self.inner.number_of_points_after_template.unwrap()
    }

    /// プロダクト定義テンプレート番号を返す。
    ///
    /// # 戻り値
    ///
    /// プロダクト定義テンプレート番号
    pub fn product_definition_template(&self) -> u16 {
        self.inner.product_definition_template.unwrap()
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
    pub fn creation_process(&self) -> u8 {
        self.inner.creation_process.unwrap()
    }

    /// 背景作成処理識別符を返す。
    ///
    /// # 戻り値
    ///
    /// 背景作成処理識別符
    pub fn background_process_identifier(&self) -> u8 {
        self.inner.background_process_identifier.unwrap()
    }

    /// 観測資料の参照時刻からの締切時間（時）を返す。
    ///
    /// # 戻り値
    ///
    /// 観測資料の参照時刻からの締切時間（時）
    pub fn deadline_hour(&self) -> u16 {
        self.inner.deadline_hour.unwrap()
    }

    /// 観測資料の参照時刻からの締切時間（分）を返す。
    ///
    /// # 戻り値
    ///
    /// 観測資料の参照時刻からの締切時間（分）
    pub fn deadline_minute(&self) -> u8 {
        self.inner.deadline_minute.unwrap()
    }

    /// 期間の単位の指示符を返す。
    ///
    /// # 戻り値
    ///
    /// 期間の単位の指示符
    pub fn term_unit_indicator(&self) -> u8 {
        self.inner.term_unit_indicator.unwrap()
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
        self.inner.first_fixed_surface_type.unwrap()
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
    pub fn stat_proc(&self) -> u8 {
        self.inner.stat_proc.unwrap()
    }

    /// 統計処理の時間増分の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 統計処理の時間増分の種類
    pub fn stat_proc_time_inc(&self) -> u8 {
        self.inner.stat_proc_time_inc.unwrap()
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
    pub fn between_successive_time_unit(&self) -> u8 {
        self.inner.between_successive_time_unit.unwrap()
    }

    /// 連続的な資料場間の時間の増分を返す。
    ///
    /// # 戻り値
    ///
    /// 連続的な資料場間の時間の増分
    pub fn between_successive_time_inc(&self) -> u32 {
        self.inner.between_successive_time_inc.unwrap()
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

    /// メソモデル予想値の結合比率の計算領域数を返す。
    ///
    /// # 戻り値
    ///
    /// メソモデル予想値の結合比率の計算領域数
    pub fn number_of_meso_model_areas(&self) -> Option<u16> {
        self.inner.number_of_meso_model_areas
    }

    /// メソモデル予想値の結合比率の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// メソモデル予想値の結合比率の尺度因子
    pub fn meso_model_ratio_scale_factor(&self) -> Option<u8> {
        self.inner.meso_model_ratio_scale_factor
    }

    /// 各領域のメソモデル予想値の結合比率を返す。
    ///
    /// # 戻り値
    ///
    /// 各領域のメソモデル予想値の結合比率
    pub fn meso_model_area_combine_ratio(&self) -> &Option<Vec<u16>> {
        &self.inner.meso_model_area_combine_ratio
    }

    /// 第5節に記録されている全資料点の数を返す。
    ///
    /// # 戻り値
    ///
    /// 第5節に記録されている全資料点の数
    pub fn total_number_of_points(&self) -> u32 {
        self.inner.total_number_of_points.unwrap()
    }

    /// 資料表現テンプレート番号を返す。
    ///
    /// # 戻り値
    ///
    /// 資料表現テンプレート番号
    pub fn data_representation_template(&self) -> u16 {
        self.inner.data_representation_template.unwrap()
    }

    /// 1データのビット数を返す。
    ///
    /// # 戻り値
    ///
    /// 1データのビット数
    pub fn bits_per_data(&self) -> u8 {
        self.inner.bits_per_data.unwrap()
    }

    /// 今回の圧縮に用いたレベルの最大値を返す。
    ///
    /// # 戻り値
    ///
    /// 圧縮に用いたレベルの最大値
    pub fn compression_max_level(&self) -> u16 {
        self.inner.compression_max_level.unwrap()
    }

    /// データの取り得るレベルの最大値を返す。
    ///
    /// # 戻り値
    ///
    /// データの取り得るレベルの最大値
    pub fn max_level(&self) -> u16 {
        self.inner.max_level.unwrap()
    }

    /// データ代表値の尺度因子を返す。
    ///
    /// # 戻り値
    ///
    /// データ代表値の尺度因子
    pub fn data_scale_factor(&self) -> u8 {
        self.inner.data_scale_factor.unwrap()
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

    /// ランレングス圧縮オクテット列の開始位置を返す。
    ///
    /// # 戻り値
    ///
    /// ランレングス圧縮オクテット列の開始位置
    pub fn run_length_position(&self) -> usize {
        self.inner.run_length_position.unwrap()
    }

    /// ランレングス圧縮オクテット列のバイト数を返す。
    ///
    /// # 戻り値
    ///
    /// ランレングス圧縮オクテット列のバイト数
    pub fn run_length_bytes(&self) -> usize {
        self.inner.run_length_bytes.unwrap()
    }

    /// ランレングス符号を走査するイテレーターを返す。
    ///
    /// # 戻り値
    ///
    /// ランレングス符号を走査するイテレーター
    pub fn values(&mut self) -> ReaderResult<Grib2ValueIter<'_>> {
        let file = File::open(self.path.as_ref())
            .map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = BufReader::new(file);
        reader
            .seek(SeekFrom::Start(self.run_length_position() as u64))
            .map_err(|_| {
                ReaderError::ReadError(
                    "ランレングス圧縮オクテット列のシークに失敗しました。".into(),
                )
            })?;

        Ok(Grib2ValueIter::new(
            reader,
            self.run_length_bytes(),
            self.number_of_points(),
            self.first_point_lat(),
            self.first_point_lon(),
            self.last_point_lon(),
            self.increment_lat(),
            self.increment_lon(),
            self.bits_per_data() as u16,
            self.compression_max_level(),
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
        writeln!(writer, "    資料分野: {}", self.document_domain())?;
        writeln!(writer, "    GRIB版番号: {}", self.grib_version())?;
        writeln!(writer, "    GRIB報全体の長さ: 0x{:08X}", self.total_bytes())?;

        Ok(())
    }

    /// 第1節:識別節を出力する。
    #[rustfmt::skip]
    pub fn write_section1<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第1節:識別節")?;
        writeln!(writer, "    作成中枢の識別: {}", self.creator_identify())?;
        writeln!(writer, "    作成副中枢: {}", self.creator_sub_identify())?;
        writeln!(writer, "    GRIBマスター表バージョン番号: {}", self.master_table_version())?;
        writeln!(writer, "    GRIB地域表バージョン番号: {}", self.local_table_version())?;
        writeln!(writer, "    参照時刻の意味: {}", self.reference_time_significance())?;
        writeln!(writer, "    資料の参照時刻: {}", self.referenced_at())?;
        writeln!(writer, "    作成ステータス: {}", self.creation_status())?;
        writeln!(writer, "    資料の種類: {}", self.document_kind())?;

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
        writeln!(writer, "    格子系定義の出典: {}", self.frame_definition_source())?;
        writeln!(writer, "    資料点数: {}", self.number_of_points())?;
        writeln!(writer, "    格子点数を定義するリストのオクテット数: {}", self.frame_system_bytes())?;
        writeln!(writer, "    格子点数を定義するリストの説明: {}", self.frame_system_description())?;
        writeln!(writer, "    格子系定義テンプレート番号: {}", self.frame_system_template())?;
        writeln!(writer, "    地球の形状: {}", self.earth_shape())?;
        writeln!(writer, "    地球回転楕円体の長軸の尺度因子: {}", self.major_axis_scale_factor())?;
        writeln!(writer, "    地球回転楕円体の長軸の尺度付きの長さ: {}", self.major_axis_length())?;
        writeln!(writer, "    地球回転楕円体の短軸の尺度因子: {}", self.minor_axis_scale_factor())?;
        writeln!(writer, "    地球回転楕円体の短軸の尺度付きの長さ: {}", self.minor_axis_length())?;
        writeln!(writer, "    緯線に沿った格子点数: {}", self.number_of_points_lat())?;
        writeln!(writer, "    経線に沿った格子点数: {}", self.number_of_points_lon())?;
        writeln!(writer, "    原作成領域の基本角: {}", self.basic_angle_area())?;
        writeln!(writer, "    最初の格子点の緯度: {}", self.first_point_lat())?;
        writeln!(writer, "    最初の格子点の経度: {}", self.first_point_lon())?;
        writeln!(writer, "    分解能及び成分フラグ: 0x{:02X}", self.resolution_and_component())?;
        writeln!(writer, "    最後の格子点の緯度: {}", self.last_point_lat())?;
        writeln!(writer, "    最後の格子点の経度: {}", self.last_point_lon())?;
        writeln!(writer, "    i方向の増分: {}", self.increment_lat())?;
        writeln!(writer, "    j方向の増分: {}", self.increment_lon())?;
        writeln!(writer, "    走査モード: 0x{:02X}", self.scan_mode())?;

        Ok(())
    }

    /// 第4節:プロダクト定義節を出力する。
    #[rustfmt::skip]
    pub fn write_section4<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第4節:プロダクト定義節")?;
        writeln!(writer, "    資料点数: {}", self.number_of_points())?;
        writeln!(writer, "    テンプレート直後の座標値の数: {}", self.number_of_points_after_template())?;
        writeln!(writer, "    プロダクト定義テンプレート番号: {}", self.product_definition_template())?;
        writeln!(writer, "    パラメータカテゴリー: {}", self.parameter_category())?;
        writeln!(writer, "    パラメータ番号: {}", self.parameter_number())?;
        writeln!(writer, "    作成処理の種類: {}", self.creation_process())?;
        writeln!(writer, "    背景作成処理識別符: {}", self.background_process_identifier())?;
        writeln!(writer, "    観測資料の参照時刻からの締切時間(時): {}", self.deadline_hour())?;
        writeln!(writer, "    観測資料の参照時刻からの締切時間(分): {}", self.deadline_minute())?;
        writeln!(writer, "    期間の単位の指示符: {}", self.term_unit_indicator())?;
        writeln!(writer, "    予報時間(分): {}", self.forecast_time())?;
        writeln!(writer, "    第一固定面の種類: {}", self.first_fixed_surface_type())?;
        writeln!(writer, "    全時間間隔の終了時: {}", self.end_of_all_time_intervals())?;
        writeln!(writer, "    統計を算出するために使用した時間間隔を記述する期間の仕様の数: {}", self.number_of_time_range_specs())?;
        writeln!(writer, "    統計処理における欠測資料の総数: {}", self.number_of_missing_values())?;
        writeln!(writer, "    統計処理の種類: {}", self.stat_proc())?;
        writeln!(writer, "    統計処理の時間増分の種類: {}", self.stat_proc_time_inc())?;
        writeln!(writer, "    統計処理の時間の単位の指示符: {}", self.stat_proc_time_unit())?;
        writeln!(writer, "    統計処理した期間の長さ: {}", self.stat_proc_time_length())?;
        writeln!(writer, "    連続的な資料場間の増分に関する時間の単位の指示符: {}", self.between_successive_time_unit())?;
        writeln!(writer, "    続的な資料場間の時間の増分: {}", self.between_successive_time_inc())?;
        writeln!(writer, "    レーダー等運用情報その1: 0x{:02X}", self.radar_info1())?;
        writeln!(writer, "    レーダー等運用情報その2: 0x{:02X}", self.radar_info2())?;
        writeln!(writer, "    雨量計運用情報: 0x{:02X}", self.rain_gauge_info())?;
        writeln!(writer, "    メソモデル予想値の結合比率の計算領域数: {:?}", self.number_of_meso_model_areas())?;
        writeln!(writer, "    メソモデル予想値の結合比率の尺度因子: {:?}", self.meso_model_ratio_scale_factor())?;
        if self.meso_model_area_combine_ratio().is_some() {
            writeln!(writer, "    各領域のメソモデル予想値の結合比率:")?;
            for (i , ratio) in self.meso_model_area_combine_ratio().as_ref().unwrap().iter().enumerate() {
                writeln!(writer, "        領域{}: {}", i + 1, ratio)?;
            }
        } else {
            writeln!(writer, "    各領域のメソモデル予想値の結合比率: None")?;
        }

        Ok(())
    }

    /// 第5節:資料表現節を出力する。
    #[rustfmt::skip]
    pub fn write_section5<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第5節:資料表現節")?;
        writeln!(writer, "    全資料点の数: {}", self.total_number_of_points())?;
        writeln!(writer, "    資料表現テンプレート番号: {}", self.data_representation_template())?;
        writeln!(writer, "    1データのビット数: {}", self.bits_per_data())?;
        writeln!(writer, "    今回の圧縮に用いたレベルの最大値: {}", self.compression_max_level())?;
        writeln!(writer, "    データの取り得るレベルの最大値: {}", self.max_level())?;
        writeln!(writer, "    データ代表値の尺度因子: {}", self.data_scale_factor())?;
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
    document_domain: Option<u8>,
    /// GRIB版番号
    grib_version: Option<u8>,
    /// GRIB報全体のバイト数
    total_bytes: Option<usize>,

    /// 第2節:識別節
    /// 作成中枢の識別
    creator_identify: Option<u16>,
    /// 作成副中枢
    creator_sub_identify: Option<u16>,
    /// GRIBマスター表バージョン番号
    master_table_version: Option<u8>,
    /// GRIB地域表バージョン番号
    local_table_version: Option<u8>,
    /// 参照時刻の意味
    reference_time_significance: Option<u8>,
    /// 資料の参照時刻
    referenced_at: Option<PrimitiveDateTime>,
    /// 作成ステータス
    creation_status: Option<u8>,
    /// 資料の種類
    document_kind: Option<u8>,

    /// 第3節:格子系定義節
    /// 格子系定義の出典
    frame_definition_source: Option<u8>,
    /// 第3節に記録されている資料点数
    number_of_points: Option<u32>,
    /// 格子点数を定義するリストのオクテット数
    frame_system_bytes: Option<u8>,
    /// 格子点数を定義するリストの説明
    frame_system_description: Option<u8>,
    /// 格子系定義テンプレート番号
    frame_system_template: Option<u16>,
    /// 地球の形状
    earth_shape: Option<u8>,
    /// 地球回転楕円体の長軸の尺度因子
    major_axis_scale_factor: Option<u8>,
    /// 地球回転楕円体の長軸の尺度付きの長さ
    major_axis_length: Option<u32>,
    /// 地球回転楕円体の短軸の尺度因子
    minor_axis_scale_factor: Option<u8>,
    /// 地球回転楕円体の短軸の尺度付きの長さ
    minor_axis_length: Option<u32>,
    /// 緯線に沿った格子点数
    number_of_points_lat: Option<u32>,
    /// 経線に沿った格子点数
    number_of_points_lon: Option<u32>,
    /// 原作成領域の基本角
    basic_angle_area: Option<u32>,
    /// 最初の格子点の緯度（10e-6度単位）
    first_point_lat: Option<u32>,
    /// 最初の格子点の経度（10e-6度単位）
    first_point_lon: Option<u32>,
    /// 最後の格子点の緯度（10e-6度単位）
    last_point_lat: Option<u32>,
    /// 分解能及び成分フラグ
    resolution_and_component: Option<u8>,
    /// 最後の格子点の経度（10e-6度単位）
    last_point_lon: Option<u32>,
    /// i方向（経度方向）の増分（10e-6度単位）
    increment_lon: Option<u32>,
    /// j方向（緯度方向）の増分（10e-6度単位）
    increment_lat: Option<u32>,
    /// 走査モード
    scan_mode: Option<u8>,

    /// 第4章:プロダクト定義節
    /// テンプレート直後の座標値の数
    number_of_points_after_template: Option<u16>,
    /// プロダクト定義テンプレート番号
    product_definition_template: Option<u16>,
    /// パラメータカテゴリー
    parameter_category: Option<u8>,
    /// パラメータ番号
    parameter_number: Option<u8>,
    /// 作成処理の種類
    creation_process: Option<u8>,
    /// 背景作成処理識別符
    background_process_identifier: Option<u8>,
    /// 観測資料の参照時刻からの締切時間（時）
    deadline_hour: Option<u16>,
    /// 観測資料の参照時刻からの締切時間（分）
    deadline_minute: Option<u8>,
    /// 期間の単位の指示符
    term_unit_indicator: Option<u8>,
    /// 予報時間
    forecast_time: Option<i32>,
    /// 第一固定面の種類
    first_fixed_surface_type: Option<u8>,
    /// 全時間間隔の終了時
    end_of_all_time_intervals: Option<PrimitiveDateTime>,
    /// 統計を算出するために使用した時間間隔を記述する期間の仕様の数
    number_of_time_range_specs: Option<u8>,
    /// 統計処理における欠測資料の総数
    number_of_missing_values: Option<u32>,
    /// 統計処理の種類
    stat_proc: Option<u8>,
    /// 統計処理の時間増分の種類
    stat_proc_time_inc: Option<u8>,
    /// 統計処理の時間の単位の指示符
    stat_proc_time_unit: Option<u8>,
    /// 統計処理した時間の長さ
    stat_proc_time_length: Option<u32>,
    /// 連続的な資料場間の増分に関する時間の単位の指示符
    between_successive_time_unit: Option<u8>,
    /// 連続的な資料場間の時間の増分
    between_successive_time_inc: Option<u32>,
    /// レーダー等運用情報その1
    radar_info1: Option<u64>,
    /// レーダー等運用情報その2
    radar_info2: Option<u64>,
    /// 雨量計運用情報
    rain_gauge_info: Option<u64>,
    /// メソモデル予想値の結合比率の計算領域数
    number_of_meso_model_areas: Option<u16>,
    /// メソモデル予想値の結合比率の尺度因子
    meso_model_ratio_scale_factor: Option<u8>,
    /// 各領域のメソモデル予想値の結合比率
    meso_model_area_combine_ratio: Option<Vec<u16>>,

    /// 第5節:資料表現節
    /// 第5節に記録されている全資料点の数
    total_number_of_points: Option<u32>,
    /// 資料表現テンプレート番号
    data_representation_template: Option<u16>,
    /// 1データのビット数
    bits_per_data: Option<u8>,
    /// 今回の圧縮に用いたレベルの最大値
    compression_max_level: Option<u16>,
    /// データの取り得るレベルの最大値
    max_level: Option<u16>,
    /// データ代表値の尺度因子
    data_scale_factor: Option<u8>,
    /// レベルmに対応するデータ代表値
    /// レベル値と物理値(mm/h)の対応を格納するコレクション
    level_values: Option<Vec<u16>>,

    /// 第6節:ビットマップ節
    /// ビットマップ指示符
    bitmap_indicator: Option<u8>,

    /// 第7節:資料値節
    /// ランレングス圧縮オクテット列の開始位置
    run_length_position: Option<usize>,
    /// ランレングス圧縮オクテット列のバイト数
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
        self.document_domain = Some(self.validate_u8(
            reader,
            DOCUMENT_DOMAIN,
            "資料分野",
            "資料分野の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // GRIB版番号: 1バイト
        self.grib_version = Some(self.validate_u8(
            reader,
            GRIB_VERSION,
            "GRIB版番号",
            "GRIB版番号の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // GRIB報全体の長さ: 8バイト
        self.total_bytes = Some(self.read_u64(reader).map_err(|_| {
            ReaderError::ReadError("第0節:GRIB報全体の長さの読み込みに失敗しました。".into())
        })? as usize);

        assert_eq!(
            16, self.read_bytes,
            "第0節読み込み終了時点で読み込んだサイズが誤っている。"
        );

        Ok(())
    }

    /// 第1節:識別節を読み込む。
    ///
    /// ファイルポインタが、第1節の開始位置にあることを想定している。
    /// 関数終了後、ファイルポインタは第3節の開始位置に移動する。
    /// なお、実装時点で、第2節は省略されている。
    fn read_section1(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        // 節の長さ: 4bytes
        self.validate_u32(
            reader,
            SECTION1_BYTES,
            "節の長さ",
            "節の長さの値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 節番号
        self.validate_u8(
            reader,
            1,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 作成中枢の識別: 2bytes
        self.creator_identify = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError("第1節:作成中枢の識別の読み込みに失敗しました。".into())
        })?);

        // 作成副中枢: 2bytes
        self.creator_sub_identify = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError("第1節:作成副中枢の読み込みに失敗しました。".into())
        })?);

        // GRIBマスター表バージョン番号: 1byte
        self.master_table_version = Some(self.validate_u8(
            reader,
            MASTER_TABLE_VERSION,
            "GRIBマスター表バージョン番号",
            "GRIBマスター表バージョン番号の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // GRIB地域表バージョン番号: 1byte
        self.local_table_version = Some(self.validate_u8(
            reader,
            LOCAL_TABLE_VERSION,
            "GRIB地域表バージョン番号",
            "GRIB地域表バージョン番号の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 参照時刻の意味: 1byte
        self.reference_time_significance = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第1節:参照時刻の意味の読み込みに失敗しました。".into())
        })?);

        // 資料の参照時刻（日時）
        self.referenced_at = Some(self.read_datetime(reader).map_err(|_| {
            ReaderError::ReadError("第1節:資料の参照時刻の読み込みに失敗しました。".into())
        })?);

        // 作成ステータス
        self.creation_status = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第1節:作成ステータスの読み込みに失敗しました。".into())
        })?);

        // 資料の種類
        self.document_kind = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第1節:資料の種類の読み込みに失敗しました。".into())
        })?);

        assert_eq!(
            37, self.read_bytes,
            "第1節読み込み終了時点で読み込んだサイズが誤っている。"
        );

        Ok(())
    }

    /// 第2節:地域使用節を読み込む。
    fn read_section2(&mut self, _reader: &mut FileReader) -> ReaderResult<()> {
        assert_eq!(
            37, self.read_bytes,
            "第2節読み込み終了時点で読み込んだサイズが誤っている。"
        );

        Ok(())
    }

    /// 第3節:格子系定義節を読み込む。
    fn read_section3(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        // 節の長さ: 4バイト
        self.validate_u32(
            reader,
            SECTION3_BYTES,
            "節の長さ",
            "節の長さの値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 節番号: 1バイト
        self.validate_u8(
            reader,
            3,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 格子系定義の出典: 1バイト
        self.frame_definition_source = Some(self.validate_u8(
            reader,
            FRAME_SYSTEM_SOURCE,
            "格子系定義の出典",
            "格子系定義の出典の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 資料点数: 4バイト
        self.number_of_points = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第3節:格子点数の読み込みに失敗しました。".into())
        })?);

        // 格子点数を定義するリストのオクテット数: 1バイト
        self.frame_system_bytes = Some(self.validate_u8(
            reader,
            0,
            "格子点数を定義するリストのオクテット数",
            "格子点数を定義するリストのオクテット数の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 格子点数を定義するリストの説明
        self.frame_system_description = Some(self.validate_u8(
            reader,
            0,
            "格子点数を定義するリストの説明",
            "格子点数を定義するリストの説明の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 格子系定義テンプレート番号: 2バイト
        self.frame_system_template = Some(self.validate_u16(
            reader,
            FRAME_SYSTEM_TEMPLATE,
            "格子系定義テンプレート番号",
            "格子系定義テンプレート番号の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 地球の形状: 1バイト
        self.earth_shape = Some(self.validate_u8(
            reader,
            EARTH_SHAPE,
            "地球の形状",
            "地球の形状の値は{value}でしたが、{expected}でなければなりません。",
        )?);

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
        self.major_axis_scale_factor = Some(self.validate_u8(
            reader,
            MAJOR_AXIS_SCALE_FACTOR,
            "地球回転楕円体の長軸の尺度因子",
            "地球回転楕円体の長軸の尺度因子の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 地球回転楕円体の長軸の尺度付きの長さ: 4バイト
        self.major_axis_length  = Some(self.validate_u32(
            reader,
            MAJOR_AXIS_LENGTH,
            "地球回転楕円体の長軸の尺度付きの長さ",
            "地球回転楕円体の長軸の尺度付きの長さの値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 地球回転楕円体の短軸の尺度因子: 1バイト
        self.minor_axis_scale_factor = Some(self.validate_u8(
            reader,
            EARTH_MINOR_AXIS_SCALE_FACTOR,
            "地球回転楕円体の短軸の尺度因子",
            "地球回転楕円体の短軸の尺度因子の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 地球回転楕円体の短軸の尺度付きの長さ: 4バイト
        self.minor_axis_length = Some(self.validate_u32(
            reader,
            EARTH_MINOR_AXIS_LENGTH,
            "地球回転楕円体の短軸の尺度付きの長さ",
            "地球回転楕円体の短軸の尺度付きの長さの値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 緯線に沿った格子点数: 4バイト
        self.number_of_points_lat = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第3節:緯線に沿った格子点数の読み込みに失敗しました。".into())
        })?);

        // 経線に沿った格子点数: 4バイト
        self.number_of_points_lon = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第3節:経線に沿った格子点数の読み込みに失敗しました。".into())
        })?);

        // 原作成領域の基本角: 4バイト
        self.basic_angle_area = Some(self.validate_u32(
            reader,
            BASIC_ANGLE_OF_AREA,
            "原作成領域の基本角",
            "原作成領域の基本角の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 端点の経度及び緯度並びに方向増分の定義に使われる基本角の細分: 4バイト
        self.seek_relative(reader, 4).map_err(|_| {
            ReaderError::ReadError(
                "第3節:端点の経度及び緯度並びに方向増分の定義に使われる基本角の細分の読み飛ばしに失敗しました。"
                    .into(),
            )
        })?;

        // 最初の格子点の緯度（10e-6度単位）: 4バイト
        self.first_point_lat = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:最初の格子点の緯度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 最初の格子点の経度（10e-6度単位）: 4バイト
        self.first_point_lon = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:最初の格子点の経度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 分解能及び成分フラグ: 1バイト
        self.resolution_and_component = Some(self.validate_u8(
            reader,
            RESOLUTION_AND_COMPONENT,
            "分解能及び成分フラグ",
            "分解能及び成分フラグの値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 最後の格子点の緯度（10e-6度単位）: 4バイト
        self.last_point_lat = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:最後の格子点の緯度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 最後の格子点の経度（10e-6度単位）: 4バイト
        self.last_point_lon = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:最後の格子点の経度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // i方向（経度方向）の増分（10e-6度単位）: 4バイト
        self.increment_lon = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:i方向（経度方向）の増分（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // j方向（緯度方向）の増分（10e-6度単位）: 4バイト
        self.increment_lat = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第3節:j方向（緯度方向）の増分（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 走査モード: 1バイト
        self.scan_mode = Some(self.validate_u8(
            reader,
            SCAN_MODE,
            "走査モード",
            "走査モードの値は{value}でしたが、{expected}でなければなりません。",
        )?);

        assert_eq!(
            109, self.read_bytes,
            "第3節読み込み終了時点で読み込んだサイズが誤っている。"
        );

        Ok(())
    }

    /// 第4節:プロダクト定義節を読み込む。
    fn read_section4(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        // 第3節までの読み込んだバイト数を記憶
        let to_section3_bytes = self.read_bytes;

        // 節の長さ: 4バイト
        let section_bytes = self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第4節:節の長さの読み込みに失敗しました。".into())
        })?;

        // 節番号: 1バイト
        self.validate_u8(
            reader,
            4,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // テンプレート直後の座標値の数: 2バイト
        self.number_of_points_after_template = Some(self.validate_u16(
            reader,
            0,
            "テンプレート直後の座標値の数",
            "テンプレート直後の座標値の数の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // プロダクト定義テンプレート番号: 2バイト
        self.product_definition_template = Some(self.read_u16(reader).map_err(|_| {
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
        self.creation_process = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:作成処理の種類の読み込みに失敗しました。".into())
        })?);

        // 背景作成処理識別符: 1バイト
        self.background_process_identifier = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:背景作成処理識別符の読み込みに失敗しました。".into())
        })?);

        // 予報の作成処理識別符: 1バイト
        self.seek_relative(reader, 1).map_err(|_| {
            ReaderError::ReadError("第4節:予報の作成処理識別符の読み飛ばしに失敗しました。".into())
        })?;

        // 観測資料の参照時刻からの締切時間（時）: 2バイト
        self.deadline_hour = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:観測資料の参照時刻からの締切時間（時）の読み込みに失敗しました。".into(),
            )
        })?);

        // 観測資料の参照時刻からの締切時間（分）: 1バイト
        self.deadline_minute = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:観測資料の参照時刻からの締切時間（分）の読み込みに失敗しました。".into(),
            )
        })?);

        // 期間の単位の指示符: 1バイト
        self.term_unit_indicator = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:期間の単位の指示符の読み込みに失敗しました。".into())
        })?);

        // 予報時間: 4バイト
        self.forecast_time = Some(self.read_i32(reader).map_err(|_| {
            ReaderError::ReadError("第4節:予報時間の読み込みに失敗しました。".into())
        })?);

        // 第一固定面の種類: 1バイト
        self.first_fixed_surface_type = Some(self.validate_u8(
            reader,
            FIRST_FIXED_SURFACE_TYPE,
            "第一固定面の種類",
            "第一固定面の種類の値は{value}でしたが、{expected}でなければなりません。",
        )?);

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
        self.number_of_time_range_specs = Some(self.validate_u8(
            reader,
            NUMBER_OF_TIME_RANGE_SPECS,
            "統計を算出するために使用した時間間隔を記述する期間の仕様の数",
            "統計を算出するために使用した時間間隔を記述する期間の仕様の数の値は{value}でしたが、{expected}でなければなりません。"
        )?);

        // 統計処理における欠測資料の総数: 4バイト
        self.number_of_missing_values = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:統計処理における欠測資料の総数の読み込みに失敗しました。".into(),
            )
        })?);

        // 統計処理の種類: 1バイト
        self.stat_proc = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第4節:統計処理の種類の読み込みに失敗しました。".into())
        })?);

        // 統計処理の時間増分の種類: 1バイト
        self.stat_proc_time_inc = Some(self.read_u8(reader).map_err(|_| {
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
        self.between_successive_time_unit = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError(
                "第4節:連続的な資料場間の増分に関する時間の単位の指示符の読み込みに失敗しました。"
                    .into(),
            )
        })?);

        // 連続的な資料場間の時間の増分: 4バイト
        self.between_successive_time_inc = Some(self.read_u32(reader).map_err(|_| {
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

        if self.product_definition_template.unwrap() == SHORT_RANGE_PRECIPITATION_FORECAST_TEMPLATE
        {
            // メソモデル予想値の結合比率の計算領域数: 2バイト
            self.number_of_meso_model_areas = Some(self.read_u16(reader).map_err(|_| {
                ReaderError::ReadError(
                    "第4節:メソモデル予想値の結合比率の計算領域数の読み込みに失敗しました。".into(),
                )
            })?);

            // メソモデル予想値の結合比率の尺度因子: 1バイト
            self.meso_model_ratio_scale_factor = Some(self.validate_u8(
                reader,
                MESO_MODEL_RATIO_SCALE_FACTOR,
                "メソモデル予想値の結合比率の尺度因子",
                "メソモデル予想値の結合比率の尺度因子の値は{value}でしたが、{expected}でなければなりません。"
            )?);

            // 各領域のメソモデル予想値の結合比率
            let mut meso_model_ratio = Vec::new();
            for _ in 0..self.number_of_meso_model_areas.unwrap() {
                meso_model_ratio.push(self.read_u16(reader).map_err(|_| {
                    ReaderError::ReadError(
                        "第4節:各領域のメソモデル予想値の結合比率の読み込みに失敗しました。".into(),
                    )
                })?);
            }
            self.meso_model_area_combine_ratio = Some(meso_model_ratio);
        }

        // 検証
        let bytes = section_bytes as i64 - (self.read_bytes - to_section3_bytes) as i64;
        if bytes < 0 {
            return Err(ReaderError::ReadError(
                format!(
                    "第4節:節の長さが不正、または読み込みに失敗しました。expected: {}, actual: {}",
                    section_bytes,
                    self.read_bytes - to_section3_bytes
                )
                .into(),
            ));
        }
        self.seek_relative(reader, bytes)
            .map_err(|_| ReaderError::ReadError("第4節の読み込みに失敗しました。".into()))?;

        Ok(())
    }

    /// 第5節:資料表現節を読み込み。
    fn read_section5(&mut self, reader: &mut FileReader) -> ReaderResult<()> {
        // 第4節までの読み込んだバイト数を記憶
        let to_section4_bytes = self.read_bytes;

        // 節の長さ: 4バイト
        let section_bytes = self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第5節:節の長さの読み込みに失敗しました。".into())
        })?;

        // 節番号: 1バイト
        self.validate_u8(
            reader,
            5,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 全資料点の数: 4バイト
        self.total_number_of_points = Some(self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第5節:全資料点の数の読み込みに失敗しました。".into())
        })?);

        // 資料表現テンプレート番号: 2バイト
        self.data_representation_template = Some(self.validate_u16(
            reader,
            DATA_REPRESENTATION_TEMPLATE,
            "資料表現テンプレート番号",
            "資料表現テンプレート番号の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 1データのビット数: 1バイト
        self.bits_per_data = Some(self.read_u8(reader).map_err(|_| {
            ReaderError::ReadError("第5節:1データのビット数の読み込みに失敗しました。".into())
        })?);

        // 今回の圧縮に用いたレベルの最大値: 2バイト
        self.compression_max_level = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError(
                "第5節:今回の圧縮に用いたレベルの最大値の読み込みに失敗しました。".into(),
            )
        })?);

        // データの取り得るレベルの最大値: 2バイト
        self.max_level = Some(self.read_u16(reader).map_err(|_| {
            ReaderError::ReadError("第5節:レベルの最大値の読み込みに失敗しました。".into())
        })?);

        // データ代表値の尺度因子: 1バイト
        self.data_scale_factor = Some(self.read_u8(reader).map_err(|_| {
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
        if section_bytes as i64 - (self.read_bytes - to_section4_bytes) as i64 != 0 {
            return Err(ReaderError::ReadError(
                format!(
                    "第4節:節の長さが不正、または読み込みに失敗しました。expected: {}, actual: {}",
                    section_bytes,
                    self.read_bytes - to_section4_bytes
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
        let section_bytes = self.read_u32(reader).map_err(|_| {
            ReaderError::ReadError("第6節:節の長さの読み込みに失敗しました。".into())
        })?;
        if section_bytes != SECTION6_BYTES {
            return Err(ReaderError::ReadError(
                format!(
                    "第6節:節の長さの読み込みに失敗しました。expected: {}, actual: {}",
                    SECTION6_BYTES, section_bytes
                )
                .into(),
            ));
        }

        // 節番号: 1バイト
        self.validate_u8(
            reader,
            6,
            "節番号",
            "節番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // ビットマップ指示符: 1バイト
        self.bitmap_indicator = Some(self.validate_u8(
            reader,
            BITMAP_INDICATOR,
            "ビットマップ指示符",
            "ビットマップ指示符の値は{value}でしたが、{expected}でなければなりません。",
        )?);

        // 検証
        if section_bytes as i64 - (self.read_bytes - to_section5_bytes) as i64 != 0 {
            return Err(ReaderError::ReadError(
                format!(
                    "第4節:節の長さが不正、または読み込みに失敗しました。expected: {}, actual: {}",
                    section_bytes,
                    self.read_bytes - to_section5_bytes
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

        // ランレングス圧縮オクテット列の開始位置を記憶
        self.run_length_position = Some(self.read_bytes);

        // ランレングス圧縮オクテット列をスキップ
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
    validate_number!(validate_u16, read_u16, u16, name, fmt);
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
/// 資料分野: 気象分野
const DOCUMENT_DOMAIN: u8 = 0;
/// GRIB版番号
const GRIB_VERSION: u8 = 2;

/// 第1節
/// 節の長さ（バイト）
const SECTION1_BYTES: u32 = 21;
/// GRIBマスター表バージョン番号
const MASTER_TABLE_VERSION: u8 = 2;
/// GRIB地域表バージョン番号
const LOCAL_TABLE_VERSION: u8 = 1;

/// 第3節
/// 節の長さ（バイト）
const SECTION3_BYTES: u32 = 72;
/// 格子系定義の出典
const FRAME_SYSTEM_SOURCE: u8 = 0;
/// 格子点定義テンプレート番号
const FRAME_SYSTEM_TEMPLATE: u16 = 0;
/// 地球の形状
const EARTH_SHAPE: u8 = 4;
/// 地球回転楕円体の長軸の尺度因子
const MAJOR_AXIS_SCALE_FACTOR: u8 = 1;
/// 地球回転楕円体の長軸の尺度付きの長さ
const MAJOR_AXIS_LENGTH: u32 = 63_781_370;
/// 地球回転楕円体の短軸の尺度因子
const EARTH_MINOR_AXIS_SCALE_FACTOR: u8 = 1;
/// 地球回転楕円体の短軸の尺度付きの長さ
const EARTH_MINOR_AXIS_LENGTH: u32 = 63_567_523;
/// 原作成領域の基本角
const BASIC_ANGLE_OF_AREA: u32 = 0;
/// 分解能及び成分フラグ
const RESOLUTION_AND_COMPONENT: u8 = 0x30;
/// 走査モード
const SCAN_MODE: u8 = 0x00;

/// 第4節
/// 1kmメッシュ降水短時間予報のプロダクト定義テンプレート番号
const SHORT_RANGE_PRECIPITATION_FORECAST_TEMPLATE: u16 = 50009;
/// 第一固定面の種類
const FIRST_FIXED_SURFACE_TYPE: u8 = 1;
/// 統計を算出するために使用した時間間隔を記述する期間の仕様の数
const NUMBER_OF_TIME_RANGE_SPECS: u8 = 1;
/// メソモデル予想値の結合比率の尺度因子
const MESO_MODEL_RATIO_SCALE_FACTOR: u8 = 0;

/// 第5節
/// 資料表現テンプレート番号
const DATA_REPRESENTATION_TEMPLATE: u16 = 200;

/// 第6節
/// 節の長さ（バイト）
const SECTION6_BYTES: u32 = 6;
/// ビットマップ指示符
const BITMAP_INDICATOR: u8 = 255;

/// 第8節
/// 第8節終端のマーカー
const SECTION8_END_MARKER: &str = "7777";
