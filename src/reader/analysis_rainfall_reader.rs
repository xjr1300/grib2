use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use num_format::{Locale, ToFormattedString};

use super::sections::{
    FromReader, Section0, Section1, Section3, Section4, Section5, Section6, Section7, Section8,
    Template3_0, Template4_50008, Template5_200, Template7_200,
};
use super::value_iter::Grib2ValueIter;
use super::{FileReader, ReaderError, ReaderResult};

pub type Section3_0 = Section3<Template3_0>;
pub type Section4_50008 = Section4<Template4_50008>;
pub type Section5_200 = Section5<Template5_200>;
pub type Section7_200 = Section7<Template7_200>;

/// 1kmメッシュ解析雨量リーダー
pub struct AnalysisRainfallReader<P>
where
    P: AsRef<Path>,
{
    /// 読み込むGRIB2ファイルのパス
    path: P,
    /// 第0節:指示節
    section0: Section0,
    /// 第1節:識別節
    section1: Section1,
    /// 第３節:格子系定義節
    section3: Section3_0,
    /// 第４節:プロダクト定義節
    section4: Section4_50008,
    /// 第５節:資料表現節
    section5: Section5_200,
    /// 第６節:ビットマップ節
    section6: Section6,
    /// 第７節:資料節
    section7: Section7_200,
    /// 第８節:終端節
    section8: Section8,
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
        let mut reader = FileReader::new(file);
        let section0 = Section0::from_reader(&mut reader)?;
        let section1 = Section1::from_reader(&mut reader)?;
        let section3 = Section3_0::from_reader(&mut reader)?;
        let section4 = Section4_50008::from_reader(&mut reader)?;
        let section5 = Section5_200::from_reader(&mut reader)?;
        let section6 = Section6::from_reader(&mut reader)?;
        let section7 = Section7_200::from_reader(&mut reader)?;
        let section8 = Section8::from_reader(&mut reader)?;

        if section3.number_of_data_points() != section5.number_of_values() {
            return Err(ReaderError::Unexpected(
                format!(
                    "第3節に記録されている資料点数({})と第5節に記録されている全資料点({})が一致しません。",
                    section3.number_of_data_points().to_formatted_string(&Locale::ja),
                    section5.number_of_values().to_formatted_string(&Locale::ja),
                ).into(),
            ));
        }

        Ok(AnalysisRainfallReader {
            path,
            section0,
            section1,
            section3,
            section4,
            section5,
            section6,
            section7,
            section8,
        })
    }

    /// 第0節:指示節を返す。
    ///
    /// # 戻り値
    ///
    /// 第0節:指示節
    pub fn section0(&self) -> &Section0 {
        &self.section0
    }

    /// 第1節:識別節を返す。
    ///
    /// # 戻り値
    ///
    /// 第1節:識別節
    pub fn section1(&self) -> &Section1 {
        &self.section1
    }

    /// 第3節:格子系定義節を返す。
    ///
    /// # 戻り値
    ///
    /// 第3節:格子系定義節
    pub fn section3(&self) -> &Section3_0 {
        &self.section3
    }

    /// 第4節:プロダクト定義節を返す。
    ///
    /// # 戻り値
    ///
    /// 第4節:プロダクト定義節
    pub fn section4(&self) -> &Section4_50008 {
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

    /// 第8節:終端節を返す。
    ///
    /// # 戻り値
    ///
    /// 第8節:終端節
    pub fn section8(&self) -> &Section8 {
        &self.section8
    }

    /// ランレングス圧縮符号を走査するイテレーターを返す。
    ///
    /// # 戻り値
    ///
    /// ランレングス圧縮符号を走査するイテレーター
    pub fn values(&mut self) -> ReaderResult<Grib2ValueIter<'_>> {
        let file = File::open(self.path.as_ref())
            .map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        reader
            .seek(SeekFrom::Start(self.section7.run_length_position() as u64))
            .map_err(|_| {
                ReaderError::ReadError("ランレングス圧縮符号列のシークに失敗しました。".into())
            })?;

        Ok(Grib2ValueIter::new(
            reader,
            self.section7.run_length_bytes(),
            self.section3.number_of_data_points(),
            self.section3.lat_of_first_grid_point(),
            self.section3.lon_of_first_grid_point(),
            self.section3.lon_of_last_grid_point(),
            self.section3.j_direction_increment(),
            self.section3.i_direction_increment(),
            self.section5.bits_per_value() as u16,
            self.section5.max_level_value(),
            self.section5.level_values(),
        ))
    }

    /// 第0節:指示節を出力する。
    #[rustfmt::skip]
    pub fn write_section0<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第0節:指示節")?;
        writeln!(writer, "    資料分野: {}", self.section0.discipline())?;
        writeln!(writer, "    GRIB版番号: {}", self.section0.edition_number())?;
        writeln!(writer, "    GRIB報全体の長さ: 0x{:08X}", self.section0.total_length())?;

        Ok(())
    }

    /// 第1節:識別節を出力する。
    #[rustfmt::skip]
    pub fn write_section1<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第1節:識別節")?;
        writeln!(writer, "    節の長さ: {}", self.section1.section_bytes())?;
        writeln!(writer, "    作成中枢の識別: {}", self.section1.center())?;
        writeln!(writer, "    作成副中枢: {}", self.section1.sub_center())?;
        writeln!(writer, "    GRIBマスター表バージョン番号: {}", self.section1.table_version())?;
        writeln!(writer, "    GRIB地域表バージョン番号: {}", self.section1.local_table_version())?;
        writeln!(writer, "    参照時刻の意味: {}", self.section1.significance_of_reference_time())?;
        writeln!(writer, "    資料の参照時刻: {}", self.section1.referenced_at())?;
        writeln!(writer, "    作成ステータス: {}", self.section1.production_status_of_processed_data())?;
        writeln!(writer, "    資料の種類: {}", self.section1.type_of_processed_data())?;

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
        writeln!(writer, "    節の長さ: {}", self.section3.section_bytes())?;
        writeln!(writer, "    格子系定義の出典: {}", self.section3.source_of_grid_definition())?;
        writeln!(writer, "    資料点数: {}", self.section3.number_of_data_points())?;
        writeln!(writer, "    格子点数を定義するリストのオクテット数: {}", self.section3.number_of_octets_for_number_of_points())?;
        writeln!(writer, "    格子点数を定義するリストの説明: {}", self.section3.interpretation_of_number_of_points())?;
        writeln!(writer, "    格子系定義テンプレート番号: {}", self.section3.grid_definition_template_number())?;
        writeln!(writer, "    地球の形状: {}", self.section3.shape_of_earth())?;
        writeln!(writer, "    地球回転楕円体の長軸の尺度因子: {}", self.section3.scale_factor_of_earth_major_axis())?;
        writeln!(writer, "    地球回転楕円体の長軸の尺度付きの長さ: {}", self.section3.scaled_value_of_earth_major_axis())?;
        writeln!(writer, "    地球回転楕円体の短軸の尺度因子: {}", self.section3.scale_factor_of_earth_minor_axis())?;
        writeln!(writer, "    地球回転楕円体の短軸の尺度付きの長さ: {}", self.section3.scaled_value_of_earth_minor_axis())?;
        writeln!(writer, "    緯線に沿った格子点数: {}", self.section3.number_of_along_lat_points())?;
        writeln!(writer, "    経線に沿った格子点数: {}", self.section3.number_of_along_lon_points())?;
        writeln!(writer, "    原作成領域の基本角: {}", self.section3.basic_angle_of_initial_product_domain())?;
        writeln!(writer, "    最初の格子点の緯度: {}", self.section3.lat_of_first_grid_point())?;
        writeln!(writer, "    最初の格子点の経度: {}", self.section3.lon_of_first_grid_point())?;
        writeln!(writer, "    分解能及び成分フラグ: 0x{:02X}", self.section3.resolution_and_component_flags())?;
        writeln!(writer, "    最後の格子点の緯度: {}", self.section3.lat_of_last_grid_point())?;
        writeln!(writer, "    最後の格子点の経度: {}", self.section3.lon_of_last_grid_point())?;
        writeln!(writer, "    i方向の増分: {}", self.section3.j_direction_increment())?;
        writeln!(writer, "    j方向の増分: {}", self.section3.i_direction_increment())?;
        writeln!(writer, "    走査モード: 0x{:02X}", self.section3.scanning_mode())?;

        Ok(())
    }

    /// 第4節:プロダクト定義節を出力する。
    #[rustfmt::skip]
    pub fn write_section4<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第4節:プロダクト定義節")?;
        writeln!(writer, "    節の長さ: {}", self.section4.section_bytes())?;
        writeln!(writer, "    テンプレート直後の座標値の数: {}", self.section4.number_of_after_template_points())?;
        writeln!(writer, "    プロダクト定義テンプレート番号: {}", self.section4.product_definition_template_number())?;
        writeln!(writer, "    パラメータカテゴリー: {}", self.section4.parameter_category())?;
        writeln!(writer, "    パラメータ番号: {}", self.section4.parameter_number())?;
        writeln!(writer, "    作成処理の種類: {}", self.section4.type_of_generating_process())?;
        writeln!(writer, "    背景作成処理識別符: {}", self.section4.background_process())?;
        writeln!(writer, "    観測資料の参照時刻からの締切時間(時): {}", self.section4.hours_after_data_cutoff())?;
        writeln!(writer, "    観測資料の参照時刻からの締切時間(分): {}", self.section4.minutes_after_data_cutoff())?;
        writeln!(writer, "    期間の単位の指示符: {}", self.section4.indicator_of_unit_of_time_range())?;
        writeln!(writer, "    予報時間（分）: {}", self.section4.forecast_time())?;
        writeln!(writer, "    第一固定面の種類: {}", self.section4.first_fixed_surface_type())?;
        writeln!(writer, "    全時間間隔の終了時: {}", self.section4.end_of_all_time_intervals())?;
        writeln!(writer, "    統計を算出するために使用した時間間隔を記述する期間の仕様の数: {}", self.section4.number_of_time_range_specs())?;
        writeln!(writer, "    統計処理における欠測資料の総数: {}", self.section4.number_of_missing_values())?;
        writeln!(writer, "    統計処理の種類: {}", self.section4.type_of_stat_proc())?;
        writeln!(writer, "    統計処理の時間増分の種類: {}", self.section4.type_of_stat_proc_time_increment())?;
        writeln!(writer, "    統計処理の時間の単位の指示符: {}", self.section4.stat_proc_time_unit())?;
        writeln!(writer, "    統計処理した期間の長さ: {}", self.section4.stat_proc_time_length())?;
        writeln!(writer, "    連続的な資料場間の増分に関する時間の単位の指示符: {}", self.section4.successive_time_unit())?;
        writeln!(writer, "    続的な資料場間の時間の増分: {}", self.section4.successive_time_increment())?;
        writeln!(writer, "    レーダー等運用情報その1: 0x{:02X}", self.section4.radar_info1())?;
        writeln!(writer, "    レーダー等運用情報その2: 0x{:02X}", self.section4.radar_info2())?;
        writeln!(writer, "    雨量計運用情報: 0x{:02X}", self.section4.rain_gauge_info())?;

        Ok(())
    }

    /// 第5節:資料表現節を出力する。
    #[rustfmt::skip]
    pub fn write_section5<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第5節:資料表現節")?;
        writeln!(writer, "    節の長さ: {}", self.section5.section_bytes())?;
        writeln!(writer, "    全資料点の数: {}", self.section5.number_of_values())?;
        writeln!(writer, "    資料表現テンプレート番号: {}", self.section5.data_representation_template_number())?;
        writeln!(writer, "    1データのビット数: {}", self.section5.bits_per_value())?;
        writeln!(writer, "    今回の圧縮に用いたレベルの最大値: {}", self.section5.max_level_value())?;
        writeln!(writer, "    データの取り得るレベルの最大値: {}", self.section5.number_of_level_values())?;
        writeln!(writer, "    データ代表値の尺度因子: {}", self.section5.decimal_scale_factor())?;
        writeln!(writer, "    レベルmに対応するデータ代表値:")?;
        for (i, level_value) in self.section5.level_values().iter().enumerate() {
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
        writeln!(writer, "    節の長さ: {}", self.section6.section_bytes())?;
        writeln!(writer, "    ビットマップ指示符数: {}", self.section6.bitmap_indicator())?;

        Ok(())
    }

    /// 第7節:資料節を出力する。
    #[rustfmt::skip]
    pub fn write_section7<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(writer, "第7節:資料節")?;
        writeln!(writer, "    節の長さ: {}", self.section7.section_bytes())?;
        writeln!(writer, "    ランレングス圧縮符号開始位置: 0x{:08X}", self.section7.run_length_position())?;
        writeln!(writer, "    ランレングス圧縮符号長さ: 0x{:08X}", self.section7.run_length_bytes())?;

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
