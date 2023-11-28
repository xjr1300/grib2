use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;
use std::str;

use time::{Date, Month, PrimitiveDateTime, Time};

/// Grib2Reader
pub struct Grib2Reader<R>
where
    R: Read + Seek,
{
    inner: Inner<R>,
}

impl Grib2Reader<File> {
    /// ファイルパスを受け取り`Grib2Reader`を構築する。
    ///
    /// # 引数
    ///
    /// * `path` - GRIB2形式のファイルのパス
    ///
    /// # 戻り値
    ///
    /// `Grib2Reader`
    pub fn new<P: AsRef<Path>>(path: P) -> ReaderResult<Self> {
        let file = File::open(path.as_ref())
            .map_err(|_| ReaderError::NotFount(path.as_ref().display().to_string().into()))?;
        let reader = BufReader::new(file);

        // 内部構造体を構築
        let mut inner = Inner::new(reader);
        // 第0節:指示節 読み込み
        inner.read_section0()?;
        // 第1節:識別節 読み込み
        inner.read_section1()?;

        Ok(Grib2Reader { inner })
    }
}

impl<R> Grib2Reader<R>
where
    R: Read + Seek,
{
    /// Readerを受け取り`Grib2Reader`を構築する。
    ///
    /// # 引数
    ///
    /// * `reader` - GRIB2形式のファイルのリーダー
    ///
    /// # 戻り値
    ///
    /// `Grib2Reader`
    pub fn new_from_reader(reader: BufReader<R>) -> ReaderResult<Self> {
        // 内部構造体を構築
        let mut inner = Inner::new(reader);
        // 第0節:指示節 読み込み
        inner.read_section0()?;
        // 第1節:識別節 読み込み
        inner.read_section1()?;

        Ok(Grib2Reader { inner })
    }

    /// GRIB報全体のバイト数を返す。
    ///
    /// # 戻り値
    ///
    /// GRIB報全体のバイト数
    pub fn total_bytes(&self) -> usize {
        self.inner.total_bytes.unwrap()
    }

    /// 資料の参照時刻を返す。
    ///
    /// # 戻り値
    ///
    /// 資料の参照時刻
    pub fn referenced_at(&self) -> PrimitiveDateTime {
        self.inner.referenced_at.unwrap()
    }

    /// 作成ステータスを返す。
    ///
    /// # 戻り値
    ///
    /// 作成ステータス
    pub fn creation_status(&self) -> u8 {
        self.inner.creation_status.unwrap()
    }

    /// 資料の種類を返す。
    ///
    /// # 戻り値
    ///
    /// 資料の種類
    pub fn document_kind(&self) -> u8 {
        self.inner.document_kind.unwrap()
    }
}

/// Grib2Readerの内部構造体
struct Inner<R>
where
    R: Read + Seek,
{
    /// GRIB2リーダー
    reader: BufReader<R>,
    /// 読み込んだバイト数
    read_bytes: usize,
    /// GRIB報全体のバイト数
    total_bytes: Option<usize>,
    /// 資料の参照時刻
    referenced_at: Option<PrimitiveDateTime>,
    /// 作成ステータス
    creation_status: Option<u8>,
    /// 資料の種類
    document_kind: Option<u8>,
}

/// `Inner`構造体が実装する符号なし整数を読み込むメソッドに展開するマクロ
macro_rules! read_uint {
    ($fname:ident, $type:ty, $size:expr) => {
        fn $fname(&mut self) -> ReaderResult<$type>
        where
            R: Read,
        {
            let mut buf = [0; $size];
            let size = self.reader.read(&mut buf).map_err(|_| {
                ReaderError::ReadError(format!("{}バイト読み込めませんでした。", $size).into())
            })?;
            if size != $size {
                return Err(ReaderError::ReadError(
                    format!("{}バイト読み込めませんでした。", $size).into(),
                ));
            }
            self.read_bytes += $size;

            Ok(<$type>::from_be_bytes(buf))
        }
    };
}

impl<R> Inner<R>
where
    R: Read + Seek,
{
    /// `Grib2Reader`の内部構造体を構築する。
    ///
    /// # 引数
    ///
    /// * `reader` - GRIB2形式のファイルのリーダー
    ///
    /// # 戻り値
    ///
    /// `Inner`
    fn new(reader: BufReader<R>) -> Self {
        Self {
            reader,
            read_bytes: 0,
            total_bytes: None,
            referenced_at: None,
            creation_status: None,
            document_kind: None,
        }
    }

    /// 第0節（指示節）を読み込み。
    ///
    /// # 引数
    ///
    /// * `reader` - ファイルリーダー
    ///
    /// # 戻り値
    ///
    /// `()`
    fn read_section0(&mut self) -> ReaderResult<()> {
        // GRIB: 4バイト
        let grib = self.read_str(4).map_err(|e| {
            ReaderError::ReadError(format!("第0節:GRIBの読み込みに失敗しました。{}", e).into())
        })?;
        if grib != "GRIB" {
            return Err(ReaderError::ReadError(
                "第0節:GRIBを読み込めませんでした。".into(),
            ));
        }

        // 保留: 2バイト
        self.seek_relative(2).map_err(|_| {
            ReaderError::ReadError("第0節:保留(5-6オクテット)の読み飛ばしに失敗しました。".into())
        })?;

        // 資料分野: 1バイト
        let discipline = self.read_u8().map_err(|_| {
            ReaderError::ReadError("第0節:資料分野の読み込みに失敗しました。".into())
        })?;
        if discipline != DOCUMENT_DOMAIN {
            return Err(ReaderError::Unexpected(
                format!(
                    "第0節:資料分野の値は{0}でなければなりません。",
                    DOCUMENT_DOMAIN
                )
                .into(),
            ));
        }

        // GRIB版番号: 1バイト
        let version = self.read_u8().map_err(|_| {
            ReaderError::ReadError("第0節:GRIB版番号の読み込みに失敗しました。".into())
        })?;
        if version != GRIB_VERSION {
            return Err(ReaderError::Unexpected(
                format!(
                    "第0節:GRIB版番号の値は{0}でなければなりません。",
                    GRIB_VERSION
                )
                .into(),
            ));
        }

        // GRIB報全体の長さ: 8バイト
        self.total_bytes = Some(self.read_u64().map_err(|_| {
            ReaderError::ReadError("第0節:GRIB報全体の長さの読み込みに失敗しました。".into())
        })? as usize);

        Ok(())
    }

    /// 第1節を読み込む。
    ///
    /// ファイルポインタが、第1節の開始位置にあることを想定している。
    /// 関数終了後、ファイルポインタは第3節の開始位置に移動する。
    /// なお、実装時点で、第2節は省略されている。
    fn read_section1(&mut self) -> ReaderResult<()> {
        // 節の長さ: 4bytes
        let section_length = self.read_u32().map_err(|_| {
            ReaderError::ReadError("第1節:節の長さの読み込みに失敗しました。".into())
        })?;
        if section_length != SECTION1_LENGTH {
            return Err(ReaderError::Unexpected(
                format!(
                    "第1節:節番号の値は{}でしたが、{}でなければなりません。",
                    section_length, SECTION1_LENGTH,
                )
                .into(),
            ));
        }

        // 節番号
        let section_number = self
            .read_u8()
            .map_err(|_| ReaderError::ReadError("第1節:節番号の読み込みに失敗しました。".into()))?;
        if section_number != 1 {
            return Err(ReaderError::Unexpected(
                format!(
                    "第1節:節番号の値は{}でしたが、1でなければなりません。",
                    section_number
                )
                .into(),
            ));
        }

        // 作成中枢の識別: 2bytes
        self.seek_relative(2).map_err(|_| {
            ReaderError::ReadError("第1節:作成中枢の識別の読み飛ばしに失敗しました。".into())
        })?;

        // 作成副中枢: 2bytes
        self.seek_relative(2).map_err(|_| {
            ReaderError::ReadError("第1節:作成副中枢の読み飛ばしに失敗しました。".into())
        })?;

        // GRIBマスター表バージョン番号: 1byte
        let master_table_version = self.read_u8().map_err(|_| {
            ReaderError::ReadError(
                "第1節:GRIBマスター表バージョン番号の読み込みに失敗しました。".into(),
            )
        })?;
        if master_table_version != MASTER_TABLE_VERSION {
            return Err(ReaderError::Unexpected(
                format!(
                    "第1節:GRIBマスター表バージョン番号は{}でしたが、{}でなければなりません。",
                    master_table_version, MASTER_TABLE_VERSION,
                )
                .into(),
            ));
        }

        // GRIB地域表バージョン番号: 1byte
        let local_table_version = self.read_u8().map_err(|_| {
            ReaderError::ReadError(
                "第1節:GRIB地域表バージョン番号の読み込みに失敗しました。".into(),
            )
        })?;
        if local_table_version != LOCAL_TABLE_VERSION {
            return Err(ReaderError::Unexpected(
                format!(
                    "第1節:GRIB地域表バージョン番号は{}でしたが、{}でなければなりません。",
                    local_table_version, LOCAL_TABLE_VERSION,
                )
                .into(),
            ));
        }

        // 参照時刻の意味: 1byte
        self.seek_relative(1).map_err(|_| {
            ReaderError::ReadError("第1節:参照時刻の意味の読み飛ばしに失敗しました。".into())
        })?;

        // 資料の参照時刻（日時）
        self.referenced_at = Some(self.read_datetime().map_err(|_| {
            ReaderError::ReadError("第1節:資料の参照時刻の読み込みに失敗しました。".into())
        })?);

        // 作成ステータス
        self.creation_status = Some(self.read_u8().map_err(|_| {
            ReaderError::ReadError("第1節:作成ステータスの読み込みに失敗しました。".into())
        })?);

        // 資料の種類
        self.document_kind = Some(self.read_u8().map_err(|_| {
            ReaderError::ReadError("第1節:資料の種類の読み込みに失敗しました。".into())
        })?);

        Ok(())
    }

    fn read_str(&mut self, size: usize) -> ReaderResult<String> {
        let mut buf = vec![0; size];
        let read_size = self.reader.read(&mut buf).map_err(|_| {
            ReaderError::ReadError(
                format!("{}バイトの文字列の読み込みに失敗しました。", size).into(),
            )
        })?;
        if read_size != size {
            return Err(ReaderError::ReadError(
                format!("{}バイトの文字列の読み込みに失敗しました。", size).into(),
            ));
        }
        self.read_bytes += size;

        Ok(String::from_utf8(buf).map_err(|_| {
            ReaderError::Unexpected(
                format!("{}バイトの文字列のコードに失敗しました。", size).into(),
            )
        }))?
    }

    fn read_datetime(&mut self) -> ReaderResult<PrimitiveDateTime> {
        let year = self.read_u16()?;
        let mut parts = Vec::new();
        for _ in 0..5 {
            parts.push(self.read_u8()?);
        }
        // 日付と時刻を構築
        let month = Month::try_from(parts[0]).map_err(|_| {
            ReaderError::Unexpected(
                format!(
                    "月の値は{}でしたが、1から12の範囲でなければなりません。",
                    parts[0]
                )
                .into(),
            )
        })?;
        let date = Date::from_calendar_date(year as i32, month, parts[1]).map_err(|_| {
            ReaderError::Unexpected(
                format!(
                    "{}年{}月は{}日を日付に変換できませんでした。",
                    year, month as u8, parts[1]
                )
                .into(),
            )
        })?;
        let time = Time::from_hms(parts[2], parts[3], parts[4]).map_err(|_| {
            ReaderError::Unexpected(
                format!(
                    "{}時{}分{}秒を時刻に変換できませんでした。",
                    parts[2], parts[3], parts[4]
                )
                .into(),
            )
        })?;

        Ok(PrimitiveDateTime::new(date, time))
    }

    read_uint!(read_u8, u8, 1);
    read_uint!(read_u16, u16, 2);
    read_uint!(read_u32, u32, 4);
    read_uint!(read_u64, u64, 8);

    /// ファイルを指定されたバイト数読み飛ばす。
    ///
    /// # 引数
    ///
    /// * `offset` - 読み飛ばすバイト数
    ///
    /// # 戻り値
    ///
    /// `()`
    fn seek_relative(&mut self, offset: i64) -> std::io::Result<()> {
        self.reader.seek_relative(offset)?;
        self.read_bytes += offset as usize;

        Ok(())
    }
}

/// 第0節
/// 資料分野: 気象分野
const DOCUMENT_DOMAIN: u8 = 0;
/// GRIB版番号
const GRIB_VERSION: u8 = 2;

/// 第1節
/// 節の長さ（バイト）
const SECTION1_LENGTH: u32 = 21;
/// GRIBマスター表バージョン番号
const MASTER_TABLE_VERSION: u8 = 2;
/// GRIB地域表バージョン番号
const LOCAL_TABLE_VERSION: u8 = 1;

#[derive(thiserror::Error, Clone, Debug)]
pub enum ReaderError {
    #[error("ファイルが見つかりません: {0}")]
    NotFount(Cow<'static, str>),
    #[error("ファイルの読み込みに失敗しました: {0}")]
    ReadError(Cow<'static, str>),
    #[error("{0}")]
    Unexpected(Cow<'static, str>),
}

pub type ReaderResult<T> = Result<T, ReaderError>;
