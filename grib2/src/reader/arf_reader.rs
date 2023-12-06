use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use num_format::{Locale, ToFormattedString};

use super::sections::{
    FromReader, Section0, Section1, Section2, Section3_0, Section4_50008, Section5_200, Section6,
    Section7_200, Section8,
};
use super::value_iter::Grib2ValueIter;
use super::{FileReader, ReaderError, ReaderResult};

/// 1kmメッシュ解析雨量リーダー
///
/// 解析雨量: Analysis Rainfall
pub struct ArfReader<P>
where
    P: AsRef<Path>,
{
    /// 読み込むGRIB2ファイルのパス
    path: P,
    /// 第0節:指示節
    section0: Section0,
    /// 第1節:識別節
    section1: Section1,
    /// 第2節:地域使用節
    section2: Section2,
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

impl<P> ArfReader<P>
where
    P: AsRef<Path>,
{
    /// ファイルパスを受け取り、解析雨量リーダーを構築する。
    ///
    /// # 引数
    ///
    /// * `path` - GRIB2形式のファイルのパス
    ///
    /// # 戻り値
    ///
    /// 解析雨量リーダー
    pub fn new(path: P) -> ReaderResult<Self> {
        let file =
            File::open(path.as_ref()).map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        let section0 = Section0::from_reader(&mut reader)?;
        let section1 = Section1::from_reader(&mut reader)?;
        let section2 = Section2::from_reader(&mut reader)?;
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

        Ok(ArfReader {
            path,
            section0,
            section1,
            section2,
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
        self.section4.debug_info(writer)?;
        writeln!(writer)?;
        self.section5.debug_info(writer)?;
        writeln!(writer)?;
        self.section6.debug_info(writer)?;
        writeln!(writer)?;
        self.section7.debug_info(writer)?;
        writeln!(writer)?;
        self.section8.debug_info(writer)?;
        writeln!(writer)?;

        Ok(())
    }
}
