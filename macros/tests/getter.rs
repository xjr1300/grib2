use macros::Getter;

#[derive(Getter)]
pub struct Foo {
    #[getter(ret = "val")]
    a: i32,
    #[getter(ret = "ref")]
    b: std::path::PathBuf,
    #[getter(ret = "ref", rty = "&str")]
    c: String,
}
