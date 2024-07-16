use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;

pub mod fprr;
mod fpsw;
mod lswj;
mod prr;
mod psw;
mod sections;
mod value;
mod value_iter;

pub use fprr::FprrReader;
pub use fpsw::FPswReader;
pub use lswj::{LswjHour, LswjReader};
pub use prr::PrrReader;
pub use psw::PswReader;
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
pub enum PswTank {
    /// 全タンク
    All = 0,
    /// 第一タンク
    First = 1,
    /// 第二タンク
    Second = 2,
}

impl std::fmt::Display for PswTank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::All => write!(f, "全タンク"),
            Self::First => write!(f, "第一タンク"),
            Self::Second => write!(f, "第二タンク"),
        }
    }
}

impl TryFrom<u8> for PswTank {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::All),
            1 => Ok(Self::First),
            2 => Ok(Self::Second),
            _ => Err("PswTankに変換できる数値は0から2までです。"),
        }
    }
}

pub(crate) fn vec_to_fixed_array<T, const N: usize>(v: Vec<T>) -> ReaderResult<[T; N]> {
    v.try_into().map_err(|v: Vec<T>| {
        ReaderError::Unexpected(
            format!(
                "配列の長さが{}である必要がありますが、{}でした。",
                N,
                v.len()
            )
            .into(),
        )
    })
}
