use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;

mod analysis_rainfall_reader;
mod grib2_value;
mod grib2_value_iter;

pub use analysis_rainfall_reader::AnalysisRainfallReader;
pub use grib2_value::Grib2Value;
pub use grib2_value_iter::Grib2ValueIter;

#[derive(thiserror::Error, Clone, Debug)]
pub enum ReaderError {
    #[error("ファイルが見つかりません: {0}")]
    NotFount(Cow<'static, str>),
    #[error("ファイルの読み込みに失敗しました: {0}")]
    ReadError(Cow<'static, str>),
    #[error("{0}")]
    Unexpected(Cow<'static, str>),
}

type FileReader = BufReader<File>;

pub type ReaderResult<T> = Result<T, ReaderError>;
