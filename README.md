# GRIB2

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
