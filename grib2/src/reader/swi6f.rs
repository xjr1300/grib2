use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use super::sections::{
    FromReader, Section0, Section1, Section2, Section3_0, Section8, SwiSections,
};
use super::{
    vec_to_fixed_array, FileReader, ForecastHour6, Grib2ValueIter, ReaderError, ReaderResult,
    SwiTank,
};

/// 土壌雨量指数6時間予想値(1km メッシュ)リーダー
/// Soil Water Index 6-hour forecast (1km mesh) reader
pub struct Swi6fReader<P>
where
    P: AsRef<Path>,
{
    path: P,
    section0: Section0,
    section1: Section1,
    section2: Section2,
    section3: Section3_0,
    forecasts: [Forecast; 6],
    section8: Section8,
}

/// 予想時間別の土壌雨量指数予想の第4節から第7節までを管理
pub struct Forecast {
    tanks: [SwiSections; 3],
}

impl Forecast {
    pub(crate) fn from_reader(reader: &mut FileReader) -> ReaderResult<Self> {
        let swi = SwiSections::from_reader(reader)?;
        let first_tank = SwiSections::from_reader(reader)?;
        let second_tank = SwiSections::from_reader(reader)?;

        Ok(Self {
            tanks: [swi, first_tank, second_tank],
        })
    }

    // 土壌雨量指数タンク別に第4節から第7節を返す。
    pub fn tank(&self, tank: SwiTank) -> &SwiSections {
        &self.tanks[tank as usize]
    }

    pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        for i in 0..3usize {
            let tank: SwiTank = (i as u8).try_into().unwrap();
            writeln!(writer, "{}:", tank)?;
            self.tank(tank).debug_info(writer)?;
            writeln!(writer)?;
        }

        Ok(())
    }
}

impl<P> Swi6fReader<P>
where
    P: AsRef<Path>,
{
    /// ファイルパスを受け取り、土壌雨量指数6時間予想値リーダーを返す。
    ///
    /// # 引数
    ///
    /// * `path` - GRIB2形式のファイルのパス
    ///
    /// # 戻り値
    ///
    /// 土壌雨量指数6時間予想値リーダー
    pub fn new(path: P) -> ReaderResult<Self> {
        let file =
            File::open(path.as_ref()).map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        let section0 = Section0::from_reader(&mut reader)?;
        let section1 = Section1::from_reader(&mut reader)?;
        let section2 = Section2::from_reader(&mut reader)?;
        let section3 = Section3_0::from_reader(&mut reader)?;
        let mut forecasts = vec![];
        for _ in 0..6 {
            forecasts.push(Forecast::from_reader(&mut reader)?);
        }
        let forecasts = vec_to_fixed_array(forecasts)?;
        let section8 = Section8::from_reader(&mut reader)?;

        Ok(Self {
            path,
            section0,
            section1,
            section2,
            section3,
            forecasts,
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

    /// 時間別の土壌雨量指数予想値の第4節から第7節までを返す。
    pub fn forecast(&self, hour: ForecastHour6) -> &Forecast {
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

    /// 予想時間別、土壌雨量指数タンク別の土壌雨量指数予想値を返すイテレーターを返す。
    pub fn forecast_value_iter(
        &mut self,
        hour: ForecastHour6,
        tank: SwiTank,
    ) -> ReaderResult<Grib2ValueIter<'_, u16>> {
        let forecast = self.forecast(hour);
        let tank = forecast.tank(tank);
        let file = File::open(self.path.as_ref())
            .map_err(|e| ReaderError::NotFount(e.to_string().into()))?;
        let mut reader = FileReader::new(file);
        reader
            .seek(SeekFrom::Start(tank.section7().run_length_position() as u64))
            .map_err(|_| {
                ReaderError::ReadError("ランレングス圧縮符号列のシークに失敗しました。".into())
            })?;

        Ok(Grib2ValueIter::new(
            reader,
            tank.section7().run_length_bytes(),
            self.section3.number_of_data_points(),
            self.section3.lat_of_first_grid_point(),
            self.section3.lon_of_first_grid_point(),
            self.section3.lon_of_last_grid_point(),
            self.section3.j_direction_increment(),
            self.section3.i_direction_increment(),
            tank.section5().bits_per_value() as u16,
            tank.section5().max_level_value(),
            tank.section5().level_values(),
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
        for i in 0..6usize {
            writeln!(writer, "{}時間後予想値:", i + 1)?;
            self.forecasts[i].debug_info(writer)?;
        }
        self.section8.debug_info(writer)?;
        writeln!(writer)?;

        Ok(())
    }
}
