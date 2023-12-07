/// GRIB2値
#[derive(Debug, Clone, Copy)]
pub struct Grib2Value<V> {
    /// 緯度（度単位）
    pub lat: f64,
    /// 経度（度単位）
    pub lon: f64,
    /// レベル値
    pub level: u16,
    /// 物理値
    /// Noneの場合は欠測値
    pub value: Option<V>,
}
