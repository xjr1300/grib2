use std::io::{Seek, SeekFrom};
use std::{fs::File, path::Path};

use super::sections::{
    FromReader, Section0, Section1, Section2, Section3, Section4, Section5, Section6, Section7,
    Section8, Template3_0, Template4_0, Template5_200, Template7_200,
};
use super::{FileReader, Grib2ValueIter, ReaderError, ReaderResult};

pub type Section3_0 = Section3<Template3_0>;
pub type Section4_0 = Section4<Template4_0>;
pub type Section5_200 = Section5<Template5_200>;
pub type Section7_200 = Section7<Template7_200>;

/// 土壌雨量指数実況値リーダー
pub struct ActualSwiReader<P>
where
    P: AsRef<Path>,
{
    path: P,
    section0: Section0,
    section1: Section1,
    section2: Section2,
    section3: Section3_0,
    /// インデックス0: 土壌雨量指数
    /// インデックス1: 第一タンク
    /// インデックス2: 第二タンク
    tanks: [ActualSwiValue; 3],
    section8: Section8,
}

fn actual_swi_value_from_reader(reader: &mut FileReader) -> ReaderResult<ActualSwiValue> {
    let section4 = Section4_0::from_reader(reader)?;
    let section5 = Section5_200::from_reader(reader)?;
    let section6 = Section6::from_reader(reader)?;
    let section7 = Section7_200::from_reader(reader)?;

    Ok(ActualSwiValue {
        section4,
        section5,
        section6,
        section7,
    })
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum Tank {
    Swi = 0,
    First = 1,
    Second = 2,
}

impl<P> ActualSwiReader<P>
where
    P: AsRef<Path>,
{
    /// ファイルパスを受け取り、土壌雨量指数実況値リーダーを返す。
    ///
    /// # 引数
    ///
    /// * `path` - GRIB2形式のファイルのパス
    ///
    /// # 戻り値
    ///
    /// 土壌雨量指数実況値リーダー
    pub fn new(path: P) -> ReaderResult<Self> {
        let file =
            File::open(path.as_ref()).map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        let section0 = Section0::from_reader(&mut reader)?;
        let section1 = Section1::from_reader(&mut reader)?;
        let section2 = Section2::from_reader(&mut reader)?;
        let section3 = Section3_0::from_reader(&mut reader)?;
        let swi = actual_swi_value_from_reader(&mut reader)?;
        let first_tank = actual_swi_value_from_reader(&mut reader)?;
        let second_tank = actual_swi_value_from_reader(&mut reader)?;
        let section8 = Section8::from_reader(&mut reader)?;

        Ok(Self {
            path,
            section0,
            section1,
            section2,
            section3,
            tanks: [swi, first_tank, second_tank],
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

    /// 土壌雨量指数を返す。
    ///
    /// # 戻り値
    ///
    /// 土壌雨量指数を記録した第4節から第7節までの節を返す。
    pub fn swi(&self) -> &ActualSwiValue {
        &self.tanks[Tank::Swi as usize]
    }

    /// 第一タンクを返す。
    ///
    /// # 戻り値
    ///
    /// 第一タンクを記録した第4節から第7節までの節を返す。
    pub fn first_tank(&self) -> &ActualSwiValue {
        &self.tanks[Tank::First as usize]
    }

    /// 第二タンクを返す。
    ///
    /// # 戻り値
    ///
    /// 第二タンクを記録した第4節から第7節までの節を返す。
    pub fn second_tank(&self) -> &ActualSwiValue {
        &self.tanks[Tank::Second as usize]
    }

    /// 第8節:終端節を返す。
    ///
    /// # 戻り値
    ///
    /// 第8節:終端節
    pub fn section8(&self) -> &Section8 {
        &self.section8
    }

    fn tank_values(&mut self, tank: Tank) -> ReaderResult<Grib2ValueIter<'_>> {
        let value_sections = &self.tanks[tank as usize];
        let file = File::open(self.path.as_ref())
            .map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        reader
            .seek(SeekFrom::Start(
                value_sections.section7.run_length_position() as u64,
            ))
            .map_err(|_| {
                ReaderError::ReadError("ランレングス圧縮符号列のシークに失敗しました。".into())
            })?;

        Ok(Grib2ValueIter::new(
            reader,
            value_sections.section7.run_length_bytes(),
            self.section3.number_of_data_points(),
            self.section3.lat_of_first_grid_point(),
            self.section3.lon_of_first_grid_point(),
            self.section3.lon_of_last_grid_point(),
            self.section3.j_direction_increment(),
            self.section3.i_direction_increment(),
            value_sections.section5.bits_per_value() as u16,
            value_sections.section5.max_level_value(),
            value_sections.section5.level_values(),
        ))
    }

    /// 土壌雨量指数を返すイテレーターを返す。
    ///
    /// # 戻り値
    ///
    /// 土壌雨量指数を返すイテレーター
    pub fn swi_values(&mut self) -> ReaderResult<Grib2ValueIter<'_>> {
        self.tank_values(Tank::Swi)
    }

    /// 第一タンクの値を返すイテレーターを返す。
    ///
    /// # 戻り値
    ///
    /// 第一タンクの値を返すイテレーター
    pub fn first_tank_values(&mut self) -> ReaderResult<Grib2ValueIter<'_>> {
        self.tank_values(Tank::First)
    }

    /// 第二タンクの値を返すイテレーターを返す。
    ///
    /// # 戻り値
    ///
    /// 第二タンクの値を返すイテレーター
    pub fn second_tank_values(&mut self) -> ReaderResult<Grib2ValueIter<'_>> {
        self.tank_values(Tank::Second)
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
        writeln!(writer, "土壌雨量指数:")?;
        self.swi().debug_info(writer)?;
        writeln!(writer)?;
        writeln!(writer, "第一タンク:")?;
        self.first_tank().debug_info(writer)?;
        writeln!(writer)?;
        writeln!(writer, "第二タンク:")?;
        self.second_tank().debug_info(writer)?;
        writeln!(writer)?;
        self.section8.debug_info(writer)?;
        writeln!(writer)?;

        Ok(())
    }
}

pub struct ActualSwiValue {
    section4: Section4_0,
    section5: Section5_200,
    section6: Section6,
    section7: Section7_200,
}

impl ActualSwiValue {
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