# GRIB2

## 読み込み可能なデータ

* 1kmメッシュ解析雨量
  * Z__C_RJTD_yyyyMMddhhmmss_SRF_GPV_Ggis1km_Prr60lv_ANAL_grib2.bin
* 1kmメッシュ降水短時間予報
  * Z__C_RJTD_yyyyMMddhhmmss_SRF_GPV_Ggis1km_Prr60lv_FH01-06_grib2.bin
* 土壌雨量指数実況値（1kmメッシュ）
  * Z__C_RJTD_yyyyMMddhhmmss_SRF_GPV_Ggis1km_Psw_Aper10min_ANAL_grib2.bin
* 土壌雨量指数6時間予測値（1kmメッシュ）
  * Z__C_RJTD_yyyyMMddhhmmss_SRF_GPV_Ggis1km_Psw_Fper10min_FH01-06_grib2.bin
* 大雨警報（土砂災害）の危険度分布（土砂災害警戒判定メッシュ情報）
  * Z__C_RJTD_yyyyMMddhhmmss_MET_INF_Jdosha_Ggis1km_ANAL_grib2.bin

## マクロ

```toml
# macros/Cargo.toml
+[[test]]
+name = "test_macros"
+path = "tests/getter.rs"
```

```sh
cargo expand --package macros --test test_macros
```

## テスト

```sh
cargo test --test analysis_rainfall -- --ignored --nocapture
```
