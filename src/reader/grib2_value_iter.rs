use std::io::Read;

use num_format::{Locale, ToFormattedString};

use super::grib2_value::Grib2Value;
use super::{FileReader, ReaderError, ReaderResult};

pub struct Grib2ValueIter<'a> {
    /// ファイルリーダー
    reader: FileReader,
    /// GRIB2ファイルに記録されている座標数
    number_of_points: u32,
    /// ランレングス圧縮オクテットを記録しているバイト数
    total_bytes: usize,
    /// 経度の最小値（10e-6度単位）
    lon_min: u32,
    /// 経度の最大値（10e-6度単位）
    lon_max: u32,
    /// 緯度の増分（10e-6度単位）
    lat_inc: u32,
    /// 経度の増分（10e-6度単位）
    lon_inc: u32,
    /// 今回のレベルの最大値
    maxv: u16,
    /// LNGU進数
    lngu: u16,
    /// レベル別物理値
    level_values: &'a [u16],
    /// ランレングス圧縮オクテットを読み込んだバイト数
    read_bytes: usize,
    /// 現在の緯度（10e-6度単位）
    current_lat: u32,
    /// 現在の経度（10e-6度単位）
    current_lon: u32,
    /// 現在のレベル値
    current_level: u16,
    /// 現在の物理値
    current_value: Option<u16>,
    /// 現在値を返却する回数
    returning_times: u32,
    /// 読み込んだ座標数
    number_of_reads: u32,
}

impl<'a> Grib2ValueIter<'a> {
    /// GRIB2値のイテレータを構築する。
    ///
    /// 引数`reader`のファイルポインタは、第7節ランレングス圧縮オクテット列の開始位置にあることを想定している。
    ///
    /// # 引数
    ///
    /// * `reader` - ファイルのリーダー
    ///
    /// # 戻り値
    ///
    /// `Grib2ValueIter`
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        reader: FileReader,
        total_bytes: usize,
        number_of_points: u32,
        lat_max: u32,
        lon_min: u32,
        lon_max: u32,
        lat_inc: u32,
        lon_inc: u32,
        nbit: u16,
        maxv: u16,
        level_values: &'a [u16],
    ) -> Self {
        Self {
            reader,
            total_bytes,
            number_of_points,
            lon_min,
            lon_max,
            lat_inc,
            lon_inc,
            maxv,
            lngu: 2u16.pow(nbit as u32) - 1 - maxv,
            level_values,
            read_bytes: 0,
            current_lat: lat_max,
            current_lon: lon_min,
            current_level: 0,
            current_value: None,
            returning_times: 0,
            number_of_reads: 0,
        }
    }

    fn read_u8(&mut self) -> ReaderResult<u8> {
        let mut buf = [0; 1];
        self.reader.read_exact(&mut buf).map_err(|_| {
            ReaderError::ReadError("ランレングス圧縮オクテットの読み込みに失敗しました。".into())
        })?;
        self.read_bytes += 1;

        Ok(u8::from_be_bytes(buf))
    }

    fn seek_relative(&mut self, offset: i64) -> ReaderResult<()> {
        self.reader.seek_relative(offset).map_err(|_| {
            ReaderError::ReadError("ランレングス圧縮オクテットのシークに失敗しました。".into())
        })?;
        self.read_bytes = (self.read_bytes as i64 + offset) as usize;

        Ok(())
    }

    fn retrieve_run_length(&mut self) -> ReaderResult<Vec<u16>> {
        let mut run_length: Vec<u16> = vec![];
        loop {
            let value = self.read_u8()? as u16;
            if value <= self.maxv && !run_length.is_empty() {
                self.seek_relative(-1)?;
                break;
            } else {
                run_length.push(value);
            }
        }

        Ok(run_length)
    }
}

impl<'a> Iterator for Grib2ValueIter<'a> {
    type Item = ReaderResult<Grib2Value>;

    fn next(&mut self) -> Option<Self::Item> {
        // 現在値返却回数が0かつ、読み込んだバイト数がランレングス圧縮符号列を記録しているバイト数に達している場合は終了
        if self.returning_times == 0 && self.total_bytes <= self.read_bytes {
            if self.number_of_reads == self.number_of_points {
                return None;
            } else {
                return Some(Err(ReaderError::Unexpected(
                    format!(
                        "読み込んだ座標数({})が第3節に記録されている資料点数({})と一致しません。\
                        ファイルが壊れている、またはクレートにバグがある可能性があります。",
                        self.number_of_reads.to_formatted_string(&Locale::ja),
                        self.number_of_points.to_formatted_string(&Locale::ja),
                    )
                    .into(),
                )));
            }
        }

        // 現在値返却回数が0の場合は、ランレングス圧縮符号を展開して現在値を更新
        if self.returning_times == 0 {
            // ランレングス圧縮符号を取得
            let run_length = self.retrieve_run_length();
            if run_length.is_err() {
                return Some(Err(run_length.err().unwrap()));
            }
            // ランレングス圧縮符号を展開
            let (level, times) = expand_run_length(&run_length.unwrap(), self.maxv, self.lngu);
            // 現在のレベル値、物理値及び返却回数を更新
            self.current_level = level;
            self.current_value = if 0 < level {
                Some(self.level_values[level as usize - 1])
            } else {
                None
            };
            self.returning_times = times;
        }

        // 結果を生成
        let result = Some(Ok(Grib2Value {
            lat: self.current_lat as f64 / 1_000_000.0,
            lon: self.current_lon as f64 / 1_000_000.0,
            level: self.current_level,
            value: self.current_value,
        }));
        // 現在値を返す回数を減らす
        self.returning_times -= 1;
        // 格子を移動
        self.current_lon += self.lon_inc;
        if self.lon_max < self.current_lon {
            self.current_lat -= self.lat_inc;
            self.current_lon = self.lon_min;
        }
        // 読み込んだ座標数をインクリメント
        self.number_of_reads += 1;

        result
    }
}

/// 1セットのランレングス符号化（圧縮）を展開する。
///
/// valuesの最初の要素はレベル値で、それ以降はランレングス値である。
/// これを1セットのランレングス符号化とする。
/// ランレングス値を含まない場合のvaluesの要素数は1で、ランレングス値を含む場合のvaluesの要素数は
/// 2以上である。
///
/// この関数が展開する、GRIB2資料テンプレート7.200（気象庁定義資料テンプレート）で利用されている
/// ランレングス符号化を以下に示す。
///
/// * 格子点値が取りうるレベル値
///   * レベル値は2次元矩形領域の格子点上に存在し、0以上MAXV以下の整数を取る。
///   * ここでMAXVは、GRIB資料表現テンプレート5.200（気象庁定義資料表現テンプレート）
///     第5節13-14オクテットで示される「今回の圧縮に用いたレベルの最大値」である。
///     * 第5節15-16オクテットの「レベルの最大値」ではないことに注意すること。
/// * 2次元データの1次元化
///   * 主走査方向を2次元矩形領域の左から右（通常西から東）、副走査方向を上から下（通常北から南）と
///     して、2次元データを1次元化する。
///     * データは最も左上の格子点の値から始まり、東方向に向かって格子点のレベル値を記録する。
///     * その緯度の最東端に達したら、下の最西端の格子点に移動して、上記同様に格子点のレベル値を記録
///       する。
///   * 最初のデータは最も左上の格子点の値であり、最後のデータは最も右下の格子点の値である。
/// * ランレングス符号化後の1格子点値当りのビット数（NBIT）
///   * NBITは、ランレングス符号化されたデータ列の中で、レベル値及びランレングス値を表現するビット数
///     である。
///   * NBITは、GRIB2資料表現テンプレート5.200第5節12オクテットで示される「1データのビット数」
///     である。
/// * 1セット内のレベル値とランレングス値の配置
///   * ランレングス符号化されたデータ列の中で0以上MAXV以下の値は各格子点のレベル値で、MAXVよりも
///     大きな値はランレングス値である。
///   * 1セットは、最初にレベル値を配置し、もしその値が連続するのであれば後ろにランレングス値を付加
///     して作成される。
///   * MAXVよりも大きな値が続く場合、それらすべては当該セットのランレングス値である。
///   * データに、MAXV以下の値が現れた時点で当該セットが終了し、このMAXV以下の値は次のセットのレベル
///     値となる。
///   * なお、同じレベル値が連続しない場合はランレングスは付加されず、次のセットに移る。
/// * ランレングス符号化方法
///   * (2 ^ NBIT - MAXV)よりも大きなランレングスが必要となった場合、1データでは表現することがで
///     きない。
///   * これに対応するために、2つ以上のランレングス値を連続させてランレングスを表現するが、連続した
///      データの単純な総和をランレングスとしても圧縮効率があがらない。
///   * よって、LNGU(=2 ^ NBIT - 1 - MAXV)進数を用いてランレングスを表現する。
///   * レベル値のすぐ後に続く最初のランレングス値(data1)をLNGU進数の1桁目
///     RL1={LNGU ^ (1 - 1) * (data1 - (MAXV + 1))}とする。
///   * それ以降n番目のランレングス値(dataN)は LNGU進数のn桁目
///     RLn={LNGU ^ (n - 1) * (dataN - (MAXV + 1))}とする。
///   * 最終的なランレングスは、それらの「総和 + 1(RL = ΣRLi + 1)」となる。
/// * ランレングス符号化例
///   * NBIT = 4、MAXV = 10とした場合、LNGU = 2 ^ 4 - 1 - 10 = 16 - 1 - 10 = 5となる。
///   * ランレングス符号化列 = {3, 9, 12, 6, 4, 15, 2, 1, 0, 13, 12, 2, 3}は、以下の通り
///     展開される。
///   * {3, 9, 9, 6, 4, 4, 4, 4, 4, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3}
///   * レベル値とランレングス値のセット{9, 12}
///     * 9がレベル値で12がランレングス値である。
///     * 12の次は6であり、10以下であるため6はレベル値である。
///     * RL1 = 5 ^ (1 - 1) * (12 - (10 + 1)) = 1 * 1 = 1
///     * RL = 1 + 1 = 2
///     * よって、9が２つ連続する。
///   * レベル値とランレングス値のセット{0, 13, 12}
///     * 0がレベル値で13と12がランレングス値である。
///     * RL1 = 5 ^ (1 - 1) * (13 - (10 + 1)) = 1 * 2 = 2
///     * RL2 = 5 ^ (2 - 1) * (12 - (10 + 1)) = 5 * 1 = 5
///     * RL = 2 + 5 + 1 = 8
///     * よって、0が8連続する。
///
/// # 引数
///
/// * `values` - 1セットのランレングス圧縮データ。
/// * `maxv` - 今回の圧縮に用いたレベルの最大値（第5節 13-14オクテット）。
/// * `lngu` - レベル値またはランレングス値のビット数をnbitとしたときの、2 ^ nbit -1 - maxvの値。
///
/// # 戻り値
///
/// レベル値とそのレベル値を繰り返す数を格納したタプル。
fn expand_run_length(values: &[u16], maxv: u16, lngu: u16) -> (u16, u32) {
    assert!(values[0] <= maxv, "values[0]={}, maxv={}", values[0], maxv);

    // ランレングス圧縮されていない場合
    if values.len() == 1 {
        return (values[0], 1);
    }

    // ランレングス圧縮を展開
    let values: Vec<u32> = values.iter().map(|v| *v as u32).collect();
    let lngu = lngu as u32;
    let maxv = maxv as u32;
    let times: u32 = values[1..]
        .iter()
        .enumerate()
        .map(|(i, &v)| lngu.pow(i as u32) * (v - (maxv + 1)))
        .sum();

    (values[0] as u16, times + 1)
}

#[cfg(test)]
mod tests {
    use super::expand_run_length;

    #[test]
    fn expand_run_length0_ok() {
        let nbit = 4;
        let maxv = 10;
        let lngu = 2u16.pow(nbit) - 1 - maxv;
        let values = vec![3u16];
        let expected = (3u16, 1u32);
        assert_eq!(expected, expand_run_length(&values, maxv, lngu));
    }

    #[test]
    fn expand_run_length1_ok() {
        let nbit = 4;
        let maxv = 10;
        let lngu = 2u16.pow(nbit) - 1 - maxv;
        let values = vec![9u16, 12];
        let expected = (9u16, 2u32);
        assert_eq!(expected, expand_run_length(&values, maxv, lngu));
    }

    #[test]
    fn expand_run_length2_ok() {
        let nbit = 4;
        let maxv = 10;
        let lngu = 2u16.pow(nbit) - 1 - maxv;
        let values = vec![4u16, 15];
        let expected = (4u16, 5u32);
        assert_eq!(expected, expand_run_length(&values, maxv, lngu));
    }

    #[test]
    fn expand_run_length3() {
        let nbit = 4;
        let maxv = 10;
        let lngu = 2u16.pow(nbit) - 1 - maxv;
        let values = vec![0u16, 13, 12];
        let expected = (0u16, 8u32);
        assert_eq!(expected, expand_run_length(&values, maxv, lngu));
    }
}
