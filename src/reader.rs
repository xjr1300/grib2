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
        let mut inner = Inner::new(reader);
        inner.read_to_section6()?;

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
        let mut inner = Inner::new(reader);
        inner.read_to_section6()?;

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

    /// 資料点数を返す。
    ///
    /// # 戻り値
    ///
    /// 資料点数
    pub fn number_of_points(&self) -> u32 {
        self.inner.number_of_points.unwrap()
    }

    /// 緯線に沿った格子点数を返す。
    ///
    /// # 戻り値
    ///
    /// 緯線に沿った格子点数
    pub fn number_of_points_lat(&self) -> u32 {
        self.inner.number_of_points_lat.unwrap()
    }

    /// 経線に沿った格子点数を返す。
    ///
    /// # 戻り値
    ///
    /// 経線に沿った格子点数
    pub fn number_of_points_lon(&self) -> u32 {
        self.inner.number_of_points_lon.unwrap()
    }

    /// 最初の格子点の緯度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最初の格子点の緯度（10e-6度単位）
    pub fn first_point_lat(&self) -> u32 {
        self.inner.first_point_lat.unwrap()
    }

    /// 最初の格子点の経度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最初の格子点の経度（10e-6度単位）
    pub fn first_point_lon(&self) -> u32 {
        self.inner.first_point_lon.unwrap()
    }

    /// 最後の格子点の緯度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最後の格子点の緯度（10e-6度単位）
    pub fn last_point_lat(&self) -> u32 {
        self.inner.last_point_lat.unwrap()
    }

    /// 最後の格子点の経度（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// 最後の格子点の経度（10e-6度単位）
    pub fn last_point_lon(&self) -> u32 {
        self.inner.last_point_lon.unwrap()
    }

    /// i方向（経度方向）の増分（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// i方向（経度方向）の増分（10e-6度単位）
    pub fn increment_lon(&self) -> u32 {
        self.inner.increment_lon.unwrap()
    }

    /// j方向（緯度方向）の増分（10e-6度単位）を返す。
    ///
    /// # 戻り値
    ///
    /// j方向（緯度方向）の増分（10e-6度単位）
    pub fn increment_lat(&self) -> u32 {
        self.inner.increment_lat.unwrap()
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
    /// 資料点数
    number_of_points: Option<u32>,
    /// 緯線に沿った格子点数
    number_of_points_lat: Option<u32>,
    /// 経線に沿った格子点数
    number_of_points_lon: Option<u32>,
    /// 最初の格子点の緯度（10e-6度単位）
    first_point_lat: Option<u32>,
    /// 最初の格子点の経度（10e-6度単位）
    first_point_lon: Option<u32>,
    /// 最後の格子点の緯度（10e-6度単位）
    last_point_lat: Option<u32>,
    /// 最後の格子点の経度（10e-6度単位）
    last_point_lon: Option<u32>,
    /// i方向（経度方向）の増分（10e-6度単位）
    increment_lon: Option<u32>,
    /// j方向（緯度方向）の増分（10e-6度単位）
    increment_lat: Option<u32>,
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

/// `Inner`構造体が実装する読み込んだ符号なし整数を検証するメソッドに展開するマクロ
macro_rules! validate_uint {
    ($fname:ident, $read_fn:ident, $type:ty, $name:ident, $fmt:ident) => {
        fn $fname(&mut self, expected: $type, $name: &str, fmt: &str) -> ReaderResult<()>
        where
            R: Read,
        {
            let value = self.$read_fn().map_err(|_| {
                ReaderError::ReadError(format!("{}の読み込みに失敗しました。", $name).into())
            })?;
            if value != expected {
                let msg = fmt
                    .replace("{value}", &value.to_string())
                    .replace("{expected}", &expected.to_string());
                return Err(ReaderError::Unexpected(msg.into()));
            }

            Ok(())
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
            number_of_points: None,
            number_of_points_lat: None,
            number_of_points_lon: None,
            first_point_lat: None,
            first_point_lon: None,
            last_point_lat: None,
            last_point_lon: None,
            increment_lon: None,
            increment_lat: None,
        }
    }

    fn read_to_section6(&mut self) -> ReaderResult<()> {
        // 第0節:指示節 読み込み
        self.read_section0()?;
        // 第1節:識別節 読み込み
        self.read_section1()?;
        // 第2節:地域使用節 読み込み
        self.read_section2()?;
        // 第3節:格子系定義節 読み込み
        self.read_section3()?;

        Ok(())
    }

    /// 第0節:指示節を読み込み。
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
        self.validate_u8(
            DOCUMENT_DOMAIN,
            "資料分野",
            "資料分野の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // GRIB版番号: 1バイト
        self.validate_u8(
            GRIB_VERSION,
            "GRIB版番号",
            "GRIB版番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // GRIB報全体の長さ: 8バイト
        self.total_bytes = Some(self.read_u64().map_err(|_| {
            ReaderError::ReadError("第0節:GRIB報全体の長さの読み込みに失敗しました。".into())
        })? as usize);

        Ok(())
    }

    /// 第1節:識別節を読み込む。
    ///
    /// ファイルポインタが、第1節の開始位置にあることを想定している。
    /// 関数終了後、ファイルポインタは第3節の開始位置に移動する。
    /// なお、実装時点で、第2節は省略されている。
    fn read_section1(&mut self) -> ReaderResult<()> {
        // 節の長さ: 4bytes
        self.validate_u32(
            SECTION1_LENGTH,
            "節の長さ",
            "節の長さの値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 節番号
        self.validate_u8(
            1,
            "節番号",
            "節番号の値は{value}でしたが、1でなければなりません。",
        )?;

        // 作成中枢の識別: 2bytes
        self.seek_relative(2).map_err(|_| {
            ReaderError::ReadError("第1節:作成中枢の識別の読み飛ばしに失敗しました。".into())
        })?;

        // 作成副中枢: 2bytes
        self.seek_relative(2).map_err(|_| {
            ReaderError::ReadError("第1節:作成副中枢の読み飛ばしに失敗しました。".into())
        })?;

        // GRIBマスター表バージョン番号: 1byte
        self.validate_u8(
            MASTER_TABLE_VERSION,
            "GRIBマスター表バージョン番号",
            "GRIBマスター表バージョン番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // GRIB地域表バージョン番号: 1byte
        self.validate_u8(
            LOCAL_TABLE_VERSION,
            "GRIB地域表バージョン番号",
            "GRIB地域表バージョン番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

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

    /// 第2節:地域使用節を読み込む。
    fn read_section2(&mut self) -> ReaderResult<()> {
        Ok(())
    }

    /// 第3節:格子系定義節を読み込む。
    fn read_section3(&mut self) -> ReaderResult<()> {
        // 節の長さ: 4バイト
        self.validate_u32(
            SECTION3_LENGTH,
            "節の長さ",
            "節の長さの値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 節番号: 1バイト
        self.validate_u8(
            3,
            "節番号",
            "節番号の値は{value}でしたが、3でなければなりません。",
        )?;

        // 格子系定義の出典: 1バイト
        self.validate_u8(
            FRAME_SYSTEM_SOURCE,
            "格子系定義の出典",
            "格子系定義の出典の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 資料点数: 4バイト
        self.number_of_points = Some(self.read_u32().map_err(|_| {
            ReaderError::ReadError("第3節:格子点数の読み込みに失敗しました。".into())
        })?);

        // 格子点数を定義するリストのオクテット数: 1バイト
        self.validate_u8(
            0,
            "格子点数を定義するリストのオクテット数",
            "格子点数を定義するリストのオクテット数の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 格子点数を定義するリストの説明
        self.validate_u8(
            0,
            "格子点数を定義するリストの説明",
            "格子点数を定義するリストの説明の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 格子系定義テンプレート番号: 2バイト
        self.validate_u16(
            FRAME_SYSTEM_TEMPLATE,
            "格子系定義テンプレート番号",
            "格子系定義テンプレート番号の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 地球の形状: 1バイト
        self.validate_u8(
            EARTH_SHAPE,
            "地球の形状",
            "地球の形状の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 地球球体の半径の尺度因子: 1バイト
        self.seek_relative(1).map_err(|_| {
            ReaderError::ReadError(
                "第3節:地球球体の半径の尺度因子の読み飛ばしに失敗しました。".into(),
            )
        })?;

        // 地球球体の尺度付き半径: 2バイト
        self.seek_relative(2).map_err(|_| {
            ReaderError::ReadError(
                "第3節:地球球体の尺度付き半径の読み飛ばしに失敗しました。".into(),
            )
        })?;

        // 地球回転楕円体の長軸の尺度因子: 1バイト
        self.validate_u8(
            EARTH_MAJOR_AXIS_SCALE_FACTOR,
            "地球回転楕円体の長軸の尺度因子",
            "地球回転楕円体の長軸の尺度因子の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 地球回転楕円体の長軸の尺度付きの長さ: 4バイト
        self.validate_u32(
            EARTH_MAJOR_AXIS_LENGTH,
            "地球回転楕円体の長軸の尺度付きの長さ",
            "地球回転楕円体の長軸の尺度付きの長さの値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 地球回転楕円体の短軸の尺度因子: 1バイト
        self.validate_u8(
            EARTH_MINOR_AXIS_SCALE_FACTOR,
            "地球回転楕円体の短軸の尺度因子",
            "地球回転楕円体の短軸の尺度因子の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 地球回転楕円体の短軸の尺度付きの長さ: 4バイト
        self.validate_u32(
            EARTH_MINOR_AXIS_LENGTH,
            "地球回転楕円体の短軸の尺度付きの長さ",
            "地球回転楕円体の短軸の尺度付きの長さの値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 緯線に沿った格子点数: 4バイト
        self.number_of_points_lat = Some(self.read_u32().map_err(|_| {
            ReaderError::ReadError("第3節:緯線に沿った格子点数の読み込みに失敗しました。".into())
        })?);

        // 経線に沿った格子点数: 4バイト
        self.number_of_points_lon = Some(self.read_u32().map_err(|_| {
            ReaderError::ReadError("第3節:経線に沿った格子点数の読み込みに失敗しました。".into())
        })?);

        // 原作成領域の基本角: 4バイト
        self.validate_u32(
            BASIC_ANGLE_OF_ORIGINAL_AREA,
            "原作成領域の基本角",
            "原作成領域の基本角の値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 端点の経度及び緯度並びに方向増分の定義に使われる基本角の細分: 4バイト
        self.seek_relative(4).map_err(|_| {
            ReaderError::ReadError(
                "第3節:端点の経度及び緯度並びに方向増分の定義に使われる基本角の細分の読み飛ばしに失敗しました。"
                    .into(),
            )
        })?;

        // 最初の格子点の緯度（10e-6度単位）: 4バイト
        self.first_point_lat = Some(self.read_u32().map_err(|_| {
            ReaderError::ReadError(
                "第3節:最初の格子点の緯度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 最初の格子点の経度（10e-6度単位）: 4バイト
        self.first_point_lon = Some(self.read_u32().map_err(|_| {
            ReaderError::ReadError(
                "第3節:最初の格子点の経度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 分解能及び成分フラグ: 1バイト
        self.validate_u8(
            RESOLUTION_AND_COMPONENT_FLAG,
            "分解能及び成分フラグ",
            "分解能及び成分フラグの値は{value}でしたが、{expected}でなければなりません。",
        )?;

        // 最後の格子点の緯度（10e-6度単位）: 4バイト
        self.last_point_lat = Some(self.read_u32().map_err(|_| {
            ReaderError::ReadError(
                "第3節:最後の格子点の緯度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 最後の格子点の経度（10e-6度単位）: 4バイト
        self.last_point_lon = Some(self.read_u32().map_err(|_| {
            ReaderError::ReadError(
                "第3節:最後の格子点の経度（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // i方向（経度方向）の増分（10e-6度単位）: 4バイト
        self.increment_lon = Some(self.read_u32().map_err(|_| {
            ReaderError::ReadError(
                "第3節:i方向（経度方向）の増分（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // j方向（緯度方向）の増分（10e-6度単位）: 4バイト
        self.increment_lat = Some(self.read_u32().map_err(|_| {
            ReaderError::ReadError(
                "第3節:j方向（緯度方向）の増分（10e-6度単位）の読み込みに失敗しました。".into(),
            )
        })?);

        // 走査モード: 1バイト
        self.validate_u8(
            SCAN_MODE,
            "走査モード",
            "走査モードの値は{value}でしたが、{expected}でなければなりません。",
        )?;

        Ok(())
    }

    // fn validate_u32(&mut self, expected: u32, fmt: &str) -> ReaderResult<()> {
    //     let value = self.read_u32()?;
    //     if value != expected {
    //         let msg = fmt
    //             .replace("{value}", &value.to_string())
    //             .replace("{expected}", &expected.to_string());
    //         return Err(ReaderError::Unexpected(msg.into()));
    //     }

    //     Ok(())
    // }

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

    validate_uint!(validate_u8, read_u8, u8, name, fmt);
    validate_uint!(validate_u16, read_u16, u16, name, fmt);
    validate_uint!(validate_u32, read_u32, u32, name, fmt);
    // validate_uint!(validate_u64, read_u64, u64, name, fmt);

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

/// 第3節
/// 節の長さ（バイト）
const SECTION3_LENGTH: u32 = 28;
/// 格子系定義の出典
const FRAME_SYSTEM_SOURCE: u8 = 0;
/// 格子点定義テンプレート番号
const FRAME_SYSTEM_TEMPLATE: u16 = 0;
/// 地球の形状
const EARTH_SHAPE: u8 = 4;
/// 地球回転楕円体の長軸の尺度因子
const EARTH_MAJOR_AXIS_SCALE_FACTOR: u8 = 1;
/// 地球回転楕円体の長軸の尺度付きの長さ
const EARTH_MAJOR_AXIS_LENGTH: u32 = 63_781_370;
/// 地球回転楕円体の短軸の尺度因子
const EARTH_MINOR_AXIS_SCALE_FACTOR: u8 = 1;
/// 地球回転楕円体の短軸の尺度付きの長さ
const EARTH_MINOR_AXIS_LENGTH: u32 = 63_567_523;
/// 原作成領域の基本角
const BASIC_ANGLE_OF_ORIGINAL_AREA: u32 = 0;
/// 分解能及び成分フラグ
const RESOLUTION_AND_COMPONENT_FLAG: u8 = 0x30;
/// 走査モード
const SCAN_MODE: u8 = 0x00;

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
