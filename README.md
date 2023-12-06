# GRIB2

## 読み込み可能なデータ

* 1kmメッシュ解析雨量
* 1kmメッシュ降水短時間予報
* 土壌雨量指数実況値（1kmメッシュ）

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
