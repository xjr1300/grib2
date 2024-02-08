use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use super::sections::{
    FromReader, Section0, Section1, Section2, Section3_0, Section4_50009, Section5_200u16,
    Section6, Section7_200, Section8,
};
use super::{FileReader, ForecastHour6, Grib2ValueIter, ReaderError, ReaderResult};

/// 1kmメッシュ降水短時間予報リーダー
pub struct FPprReader<P>
where
    P: AsRef<Path>,
{
    path: P,
    section0: Section0,
    section1: Section1,
    section2: Section2,
    section3: Section3_0,
    forecasts: [FPprSections; 6],
    section8: Section8,
}

pub struct FPprSections {
    section4: Section4_50009,
    section5: Section5_200u16,
    section6: Section6,
    section7: Section7_200,
}

impl FPprSections {
    fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        let section4 = Section4_50009::from_reader(reader)?;
        let section5 = Section5_200u16::from_reader(reader)?;
        let section6 = Section6::from_reader(reader)?;
        let section7 = Section7_200::from_reader(reader)?;

        Ok(FPprSections {
            section4,
            section5,
            section6,
            section7,
        })
    }
}

/// 時間別の降水短時間予想値を返すイテレーターを返すメソッドを生成するマクロ
macro_rules! impl_forecast_values_iter {
    ($([$fn:ident, $hour:ident]),*) => {
        $(
            pub fn $fn(&mut self) -> ReaderResult<Grib2ValueIter<'_, u16>> {
                self.forecast_value_iter(ForecastHour6::$hour)
            }
        )*
    }
}

impl<P> FPprReader<P>
where
    P: AsRef<Path>,
{
    pub fn new(path: P) -> ReaderResult<Self> {
        let file =
            File::open(path.as_ref()).map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        let section0 = Section0::from_reader(&mut reader)?;
        let section1 = Section1::from_reader(&mut reader)?;
        let section2 = Section2::from_reader(&mut reader)?;
        let section3 = Section3_0::from_reader(&mut reader)?;
        let hour1 = FPprSections::from_reader(&mut reader)?;
        let hour2 = FPprSections::from_reader(&mut reader)?;
        let hour3 = FPprSections::from_reader(&mut reader)?;
        let hour4 = FPprSections::from_reader(&mut reader)?;
        let hour5 = FPprSections::from_reader(&mut reader)?;
        let hour6 = FPprSections::from_reader(&mut reader)?;
        let section8 = Section8::from_reader(&mut reader)?;

        Ok(Self {
            path,
            section0,
            section1,
            section2,
            section3,
            forecasts: [hour1, hour2, hour3, hour4, hour5, hour6],
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

    // 時間別の降水短時間予報を返す。
    pub fn forecast(&self, hour: ForecastHour6) -> &FPprSections {
        &self.forecasts[hour as usize - 1]
    }

    /// 第8節:気象要素値節を返す。
    ///
    /// # 戻り値
    ///
    /// 第8節:気象要素値節
    pub fn section8(&self) -> &Section8 {
        &self.section8
    }

    /// 時間別の降水短時間予想値を返すイテレーターを返す。
    pub fn forecast_value_iter(
        &mut self,
        hour: ForecastHour6,
    ) -> ReaderResult<Grib2ValueIter<'_, u16>> {
        let forecast = &self.forecasts[hour as usize - 1];
        let file = File::open(self.path.as_ref())
            .map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        reader
            .seek(SeekFrom::Start(
                forecast.section7.run_length_position() as u64
            ))
            .map_err(|_| {
                ReaderError::ReadError("ランレングス圧縮符号列のシークに失敗しました。".into())
            })?;

        Ok(Grib2ValueIter::new(
            reader,
            forecast.section7.run_length_bytes(),
            self.section3.number_of_data_points(),
            self.section3.lat_of_first_grid_point(),
            self.section3.lon_of_first_grid_point(),
            self.section3.lon_of_last_grid_point(),
            self.section3.j_direction_increment(),
            self.section3.i_direction_increment(),
            forecast.section5.bits_per_value() as u16,
            forecast.section5.max_level_value(),
            forecast.section5.level_values(),
        ))
    }

    // 時間別の降水短時間予想値を返すイテレーターを返すメソッド
    impl_forecast_values_iter!(
        [forecast_hour1_value_iter, Hour1],
        [forecast_hour2_value_iter, Hour2],
        [forecast_hour3_value_iter, Hour3],
        [forecast_hour4_value_iter, Hour4],
        [forecast_hour5_value_iter, Hour5],
        [forecast_hour6_value_iter, Hour6]
    );

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
        for i in 0..6usize {
            writeln!(writer, "{}時間後予想値:", i + 1)?;
            self.forecasts[i].debug_info(writer)?;
        }
        self.section8.debug_info(writer)?;
        writeln!(writer)?;

        Ok(())
    }
}

impl FPprSections {
    /// 第4節:格子点値節を返す。
    ///
    /// # 戻り値
    ///
    /// 第4節:格子点値節
    pub fn section4(&self) -> &Section4_50009 {
        &self.section4
    }

    /// 第5節:気象要素値節を返す。
    ///
    /// # 戻り値
    ///
    /// 第5節:気象要素値節
    pub fn section5(&self) -> &Section5_200u16 {
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

    /// 第7節:気象要素値節を返す。
    ///
    /// # 戻り値
    ///
    /// 第7節:気象要素値節
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
        writeln!(writer)?;

        Ok(())
    }
}
