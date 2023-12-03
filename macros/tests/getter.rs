use macros::{Getter, TemplateGetter};

#[derive(Getter)]
pub struct Foo {
    #[getter(ret = "val")]
    a: i32,
    #[getter(ret = "ref")]
    b: std::path::PathBuf,
    #[getter(ret = "ref", rty = "&str")]
    c: String,
}

pub struct Section2<T2> {
    template2: T2,
}

#[derive(TemplateGetter)]
#[template_getter(section = "Section2", member = "template2")]
pub struct Template2 {
    #[getter(ret = "val")]
    a: i32,
    #[getter(ret = "ref")]
    b: std::path::PathBuf,
    #[getter(ret = "ref", ty = "&str")]
    c: String,
}
