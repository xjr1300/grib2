use std::io::{Seek, SeekFrom};
use std::{fs::File, path::Path};

use super::sections::{
    FromReader, PswSections, Section0, Section1, Section2, Section3_0, Section8,
};
use super::{FileReader, Grib2ValueIter, PswTank, ReaderError, ReaderResult};

/// 土壌雨量指数ファイルリーダー
pub struct PswReader<P>
where
    P: AsRef<Path>,
{
    /// ファイルのパス
    path: P,
    /// 第4節プロダクト定義節読み込みフラグ
    should_read_product_def: bool,

    section0: Section0,
    section1: Section1,
    section2: Section2,
    section3: Section3_0,
    /// インデックス0: 土壌雨量指数
    /// インデックス1: 第一タンク
    /// インデックス2: 第二タンク
    tanks: [PswSections; 3],
    section8: Section8,
}

impl<P> PswReader<P>
where
    P: AsRef<Path>,
{
    /// ファイルパスを受け取り、土壌雨量指数実況値リーダーを返す。
    ///
    /// # 引数
    ///
    /// * `path` - GRIB2形式のファイルのパス
    /// * `should_read_product_def` - 第4節プロダクト定義節読み込みフラグ
    ///
    /// # 戻り値
    ///
    /// 土壌雨量指数実況値リーダー
    pub fn new(path: P, should_read_product_def: bool) -> ReaderResult<Self> {
        let file =
            File::open(path.as_ref()).map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        let section0 = Section0::from_reader(&mut reader)?;
        let section1 = Section1::from_reader(&mut reader)?;
        let section2 = Section2::from_reader(&mut reader)?;
        let section3 = Section3_0::from_reader(&mut reader)?;
        let all_tanks = PswSections::from_reader(&mut reader, should_read_product_def)?;
        let first_tank = PswSections::from_reader(&mut reader, should_read_product_def)?;
        let second_tank = PswSections::from_reader(&mut reader, should_read_product_def)?;
        let section8 = Section8::from_reader(&mut reader)?;

        Ok(Self {
            path,
            should_read_product_def,
            section0,
            section1,
            section2,
            section3,
            tanks: [all_tanks, first_tank, second_tank],
            section8,
        })
    }

    /// 第4節プロダクト定義節読み込みフラグを返す。
    ///
    /// # 戻り値
    ///
    /// 第4節プロダクト定義節を読み込む場合はtrue、それ以外はfalse。
    pub fn should_read_product_def(&self) -> bool {
        self.should_read_product_def
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

    /// 第2節:地域使用節を返す。
    ///
    /// # 戻り値
    ///
    /// 第2節:地域使用節
    pub fn section2(&self) -> &Section2 {
        &self.section2
    }

    /// 第3節:格子系定義節を返す。
    ///
    /// # 戻り値
    ///
    /// 第3節:格子系定義節
    pub fn section3(&self) -> &Section3_0 {
        &self.section3
    }

    /// 全タンクを返す。
    ///
    /// # 戻り値
    ///
    /// 土壌雨量指数を記録した第4節から第7節までの節を返す。
    pub fn all_tanks_sections(&self) -> &PswSections {
        &self.tanks[PswTank::All as usize]
    }

    /// 第一タンクを返す。
    ///
    /// # 戻り値
    ///
    /// 第一タンクを記録した第4節から第7節までの節を返す。
    pub fn first_tank_sections(&self) -> &PswSections {
        &self.tanks[PswTank::First as usize]
    }

    /// 第二タンクを返す。
    ///
    /// # 戻り値
    ///
    /// 第二タンクを記録した第4節から第7節までの節を返す。
    pub fn second_tank_sections(&self) -> &PswSections {
        &self.tanks[PswTank::Second as usize]
    }

    /// 第8節:終端節を返す。
    ///
    /// # 戻り値
    ///
    /// 第8節:終端節
    pub fn section8(&self) -> &Section8 {
        &self.section8
    }

    fn value_iter(&mut self, tank: PswTank) -> ReaderResult<Grib2ValueIter<'_, u16>> {
        let value_sections = &self.tanks[tank as usize];
        let file = File::open(self.path.as_ref())
            .map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        reader
            .seek(SeekFrom::Start(
                value_sections.section7().run_length_position() as u64,
            ))
            .map_err(|_| {
                ReaderError::ReadError("ランレングス圧縮符号列のシークに失敗しました。".into())
            })?;

        Ok(Grib2ValueIter::new(
            reader,
            value_sections.section7().run_length_bytes(),
            self.section3.number_of_data_points(),
            self.section3.lat_of_first_grid_point(),
            self.section3.lon_of_first_grid_point(),
            self.section3.lon_of_last_grid_point(),
            self.section3.j_direction_increment(),
            self.section3.i_direction_increment(),
            value_sections.section5().bits_per_value() as u16,
            value_sections.section5().max_level_value(),
            value_sections.section5().level_values(),
        ))
    }

    /// 全タンクの値を返すイテレーターを返す。
    ///
    /// # 戻り値
    ///
    /// 全タンクの値を返すイテレーター
    pub fn all_tanks_value_iter(&mut self) -> ReaderResult<Grib2ValueIter<'_, u16>> {
        self.value_iter(PswTank::All)
    }

    /// 第一タンクの値を返すイテレーターを返す。
    ///
    /// # 戻り値
    ///
    /// 第一タンクの値を返すイテレーター
    pub fn first_tank_value_iter(&mut self) -> ReaderResult<Grib2ValueIter<'_, u16>> {
        self.value_iter(PswTank::First)
    }

    /// 第二タンクの値を返すイテレーターを返す。
    ///
    /// # 戻り値
    ///
    /// 第二タンクの値を返すイテレーター
    pub fn second_tank_value_iter(&mut self) -> ReaderResult<Grib2ValueIter<'_, u16>> {
        self.value_iter(PswTank::Second)
    }

    /// 全ての節を出力する。
    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.section0.debug_info(writer)?;
        writeln!(writer)?;
        self.section1.debug_info(writer)?;
        writeln!(writer)?;
        self.section2.debug_info(writer)?;
        writeln!(writer)?;
        self.section3.debug_info(writer)?;
        writeln!(writer)?;
        writeln!(writer, "全タンク:")?;
        self.all_tanks_sections().debug_info(writer)?;
        writeln!(writer)?;
        writeln!(writer, "第一タンク:")?;
        self.first_tank_sections().debug_info(writer)?;
        writeln!(writer)?;
        writeln!(writer, "第二タンク:")?;
        self.second_tank_sections().debug_info(writer)?;
        writeln!(writer)?;
        self.section8.debug_info(writer)?;
        writeln!(writer)?;

        Ok(())
    }
}
