use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;

mod arf_reader;
mod aswi_reader;
mod sections;
mod value;
mod value_iter;

pub use arf_reader::ArfReader;
pub use aswi_reader::AswiReader;
pub use value::Grib2Value;
pub use value_iter::Grib2ValueIter;

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
