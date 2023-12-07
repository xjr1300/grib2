use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;

mod arf_reader;
mod aswi_reader;
mod sections;
pub mod srpf_reader;
mod swi6f_reader;
mod value;
mod value_iter;

pub use arf_reader::ArfReader;
pub use aswi_reader::AswiReader;
pub use srpf_reader::SrpfReader;
pub use swi6f_reader::Swi6fReader;
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

/// 6時間予報時間
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ForecastHour6 {
    /// 1時間後予報
    Hour1 = 1,
    /// 2時間後予報
    Hour2 = 2,
    /// 3時間後予報
    Hour3 = 3,
    /// 4時間後予報
    Hour4 = 4,
    /// 5時間後予報
    Hour5 = 5,
    /// 6時間後予報
    Hour6 = 6,
}

impl std::fmt::Display for ForecastHour6 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hour = match self {
            Self::Hour1 => 1,
            Self::Hour2 => 2,
            Self::Hour3 => 3,
            Self::Hour4 => 4,
            Self::Hour5 => 5,
            Self::Hour6 => 6,
        };

        write!(f, "{}時間後予報", hour)
    }
}

impl TryFrom<u8> for ForecastHour6 {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Hour1),
            2 => Ok(Self::Hour2),
            3 => Ok(Self::Hour3),
            4 => Ok(Self::Hour4),
            5 => Ok(Self::Hour5),
            6 => Ok(Self::Hour6),
            _ => Err("ForecastHourに変換できる数値は1から6までです。"),
        }
    }
}

/// 土壌雨量指数タンク
#[repr(C)]
pub enum SwiTank {
    /// 土壌雨量指数
    Swi = 0,
    /// 第一タンク
    First = 1,
    /// 第二タンク
    Second = 2,
}

impl std::fmt::Display for SwiTank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Swi => write!(f, "土壌雨量指数"),
            Self::First => write!(f, "第一タンク"),
            Self::Second => write!(f, "第二タンク"),
        }
    }
}

impl TryFrom<u8> for SwiTank {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Swi),
            1 => Ok(Self::First),
            2 => Ok(Self::Second),
            _ => Err("SwiTankに変換できる数値は0から2までです。"),
        }
    }
}
