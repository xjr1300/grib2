use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use super::sections::{
    FromReader, Section0, Section1, Section2, Section3_0, Section4_50009, Section5_200, Section6,
    Section7_200, Section8,
};
use super::{FileReader, Grib2ValueIter, ReaderError, ReaderResult};

/// 降水短時間予報リーダー
///
/// 降水短時間予報: Short Range Precipitation Forecast
pub struct SrpfReader<P>
where
    P: AsRef<Path>,
{
    path: P,
    section0: Section0,
    section1: Section1,
    section2: Section2,
    section3: Section3_0,
    forecast_sections: [SrpfForecast; 6],
    section8: Section8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ForecastHour {
    Hour1 = 1,
    Hour2 = 2,
    Hour3 = 3,
    Hour4 = 4,
    Hour5 = 5,
    Hour6 = 6,
}

impl TryFrom<i32> for ForecastHour {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ForecastHour::Hour1),
            2 => Ok(ForecastHour::Hour2),
            3 => Ok(ForecastHour::Hour3),
            4 => Ok(ForecastHour::Hour4),
            5 => Ok(ForecastHour::Hour5),
            6 => Ok(ForecastHour::Hour6),
            _ => Err("ForecastHourに変換できる数値は1から6までです。"),
        }
    }
}

pub struct SrpfForecast {
    section4: Section4_50009,
    section5: Section5_200,
    section6: Section6,
    section7: Section7_200,
}

/// 時間別の降水短時間予報を返すメソッドを生成するマクロ
macro_rules! impl_forecast_hours {
    ($([$fn:ident, $hour:ident]),*) => {
        $(
            pub fn $fn(&self) -> &SrpfForecast {
                &self.forecast_sections[ForecastHour::$hour as usize - 1]
            }
        )*
    }
}

/// 時間別の降水短時間予測値を返すイテレーターを返すメソッドを生成するマクロ
macro_rules! impl_forecast_values_iter {
    ($([$fn:ident, $hour:ident]),*) => {
        $(
            pub fn $fn(&mut self) -> ReaderResult<Grib2ValueIter<'_>> {
                self.forecast_value_iter(ForecastHour::$hour)
            }
        )*
    }
}

impl<P> SrpfReader<P>
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
        let hour1 = srpf_forecast_from_reader(&mut reader)?;
        let hour2 = srpf_forecast_from_reader(&mut reader)?;
        let hour3 = srpf_forecast_from_reader(&mut reader)?;
        let hour4 = srpf_forecast_from_reader(&mut reader)?;
        let hour5 = srpf_forecast_from_reader(&mut reader)?;
        let hour6 = srpf_forecast_from_reader(&mut reader)?;
        let section8 = Section8::from_reader(&mut reader)?;

        Ok(Self {
            path,
            section0,
            section1,
            section2,
            section3,
            forecast_sections: [hour1, hour2, hour3, hour4, hour5, hour6],
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
    impl_forecast_hours!(
        [hour1, Hour1],
        [hour2, Hour2],
        [hour3, Hour3],
        [hour4, Hour4],
        [hour5, Hour5],
        [hour6, Hour6]
    );

    /// 第8節:気象要素値節を返す。
    ///
    /// # 戻り値
    ///
    /// 第8節:気象要素値節
    pub fn section8(&self) -> &Section8 {
        &self.section8
    }

    pub fn forecast_value_iter(&mut self, hour: ForecastHour) -> ReaderResult<Grib2ValueIter<'_>> {
        let forecast = &self.forecast_sections[hour as usize - 1];
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

    // 時間別の降水短時間予測値を返すイテレーターを返すメソッド
    impl_forecast_values_iter!(
        [forecast_hour1, Hour1],
        [forecast_hour2, Hour2],
        [forecast_hour3, Hour3],
        [forecast_hour4, Hour4],
        [forecast_hour5, Hour5],
        [forecast_hour6, Hour6]
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
            self.forecast_sections[i].debug_info(writer)?;
        }
        self.section8.debug_info(writer)?;
        writeln!(writer)?;

        Ok(())
    }
}

fn srpf_forecast_from_reader(reader: &mut FileReader) -> ReaderResult<SrpfForecast> {
    let section4 = Section4_50009::from_reader(reader)?;
    let section5 = Section5_200::from_reader(reader)?;
    let section6 = Section6::from_reader(reader)?;
    let section7 = Section7_200::from_reader(reader)?;

    Ok(SrpfForecast {
        section4,
        section5,
        section6,
        section7,
    })
}

impl SrpfForecast {
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
