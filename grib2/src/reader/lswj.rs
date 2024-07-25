use std::io::{Seek, SeekFrom};
use std::{fs::File, path::Path};

use super::sections::{
    FromReader, Section0, Section1, Section2, Section3_0, Section4_50000, Section5_200i16,
    Section6, Section7_200, Section8,
};
use super::{FileReader, Grib2ValueIter, ReaderError, ReaderResult};

/// 実況及び3時間先までの土砂災害警戒判定リーダー
///
/// 土砂災害警戒判定: Landslide Warning Judgement
pub struct LswjReader<P>
where
    P: AsRef<Path>,
{
    path: P,
    section0: Section0,
    section1: Section1,
    section2: Section2,
    section3: Section3_0,
    judgments: [LswjSections; 4],
    section8: Section8,
}

pub struct LswjSections {
    section4: Section4_50000,
    section5: Section5_200i16,
    section6: Section6,
    section7: Section7_200,
}

impl LswjSections {
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        let section4 = Section4_50000::from_reader(reader)?;
        let section5 = Section5_200i16::from_reader(reader)?;
        let section6 = Section6::from_reader(reader)?;
        let section7 = Section7_200::from_reader(reader)?;

        Ok(Self {
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
    pub fn section4(&self) -> &Section4_50000 {
        &self.section4
    }

    /// 第5節:資料表現節を返す。
    ///
    /// # 戻り値
    ///
    /// 第5節:資料表現節
    pub fn section5(&self) -> &Section5_200i16 {
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
        self.section4().debug_info(writer)?;
        writeln!(writer)?;
        self.section5().debug_info(writer)?;
        writeln!(writer)?;
        self.section6().debug_info(writer)?;
        writeln!(writer)?;
        self.section7().debug_info(writer)?;
        writeln!(writer)?;

        Ok(())
    }
}

impl<P> LswjReader<P>
where
    P: AsRef<Path>,
{
    /// ファイルパスを受け取り、土砂災害警戒判定メッシュリーダーを構築する。
    ///
    /// # 引数
    ///
    /// * `path` - GRIB2形式のファイルのパス
    ///
    /// # 戻り値
    ///
    /// 土砂災害警戒判定メッシュリーダー
    pub fn new(path: P) -> ReaderResult<Self> {
        let file =
            File::open(path.as_ref()).map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        let section0 = Section0::from_reader(&mut reader)?;
        let section1 = Section1::from_reader(&mut reader)?;
        let section2 = Section2::from_reader(&mut reader)?;
        let section3 = Section3_0::from_reader(&mut reader)?;
        let actual = LswjSections::from_reader(&mut reader)?;
        let hour1 = LswjSections::from_reader(&mut reader)?;
        let hour2 = LswjSections::from_reader(&mut reader)?;
        let hour3 = LswjSections::from_reader(&mut reader)?;
        let section8 = Section8::from_reader(&mut reader)?;

        Ok(LswjReader {
            path,
            section0,
            section1,
            section2,
            section3,
            judgments: [actual, hour1, hour2, hour3],
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

    /// 第4節から第7節を返す。
    ///
    /// # 戻り値
    ///
    /// 第4節から第7節
    pub fn judgment(&self, hour: LswjHour) -> &LswjSections {
        &self.judgments[hour as usize]
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
    pub fn values(&mut self, hour: LswjHour) -> ReaderResult<Grib2ValueIter<'_, i16>> {
        let judgment = &self.judgments[hour as usize];
        let file = File::open(self.path.as_ref())
            .map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        reader
            .seek(SeekFrom::Start(
                judgment.section7.run_length_position() as u64
            ))
            .map_err(|_| {
                ReaderError::ReadError("ランレングス圧縮符号列のシークに失敗しました。".into())
            })?;

        Ok(Grib2ValueIter::new(
            reader,
            judgment.section7.run_length_bytes(),
            self.section3.number_of_data_points(),
            self.section3.lat_of_first_grid_point(),
            self.section3.lon_of_first_grid_point(),
            self.section3.lon_of_last_grid_point(),
            self.section3.j_direction_increment(),
            self.section3.i_direction_increment(),
            judgment.section5.bits_per_value() as u16,
            judgment.section5.max_level_value(),
            judgment.section5.level_values(),
        ))
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
        for i in 0..=3usize {
            writeln!(writer, "{}", LswjHour::try_from(i).unwrap())?;
            self.judgments[i].debug_info(writer)?;
        }
        self.section8.debug_info(writer)?;
        writeln!(writer)?;

        Ok(())
    }
}

/// 土砂災害警戒判定時間
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LswjHour {
    Actual = 0,
    Hour1 = 1,
    Hour2 = 2,
    Hour3 = 3,
}

impl std::fmt::Display for LswjHour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Actual => write!(f, "実況"),
            Self::Hour1 => write!(f, "1時間予想"),
            Self::Hour2 => write!(f, "2時間予想"),
            Self::Hour3 => write!(f, "3時間予想"),
        }
    }
}

impl TryFrom<usize> for LswjHour {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Actual),
            1 => Ok(Self::Hour1),
            2 => Ok(Self::Hour2),
            3 => Ok(Self::Hour3),
            _ => Err("can not parse to LswjHour"),
        }
    }
}
