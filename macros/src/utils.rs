use syn::punctuated::Punctuated;
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Expr, Field, Fields, FieldsNamed, Lit, MetaNameValue,
    Token,
};

pub(crate) type CommaPunctuatedFields = Punctuated<Field, Token![,]>;
pub(crate) type CommaPunctuatedNameValues = Punctuated<MetaNameValue, Token![,]>;

/// 構造体のフィールドを取得する。
pub(crate) fn retrieve_struct_fields(input: &DeriveInput) -> syn::Result<CommaPunctuatedFields> {
    match input.clone().data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => Ok(named),
        _ => Err(syn::Error::new_spanned(input, "expected struct")),
    }
}

pub(crate) struct FieldAttrPair<'a> {
    /// フィールド
    pub field: &'a Field,
    /// 属性
    pub attr: &'a Attribute,
}

/// 指定された属性が付与されたフィールドとその属性を取得する。
pub(crate) fn retrieve_field_attrs_by_names<'a>(
    fields: &'a CommaPunctuatedFields,
    names: &[&str],
) -> Vec<FieldAttrPair<'a>> {
    // 指定された属性が付与されたフィールドとその属性を格納するベクタ
    let mut result = vec![];
    // 構造体のフィールドを走査
    for field in fields {
        field.attrs.iter().for_each(|attr| {
            // 指定された属性が付与されたフィールドか確認
            for name in names {
                if attr.path().is_ident(name) {
                    result.push(FieldAttrPair { field, attr });
                }
            }
        });
    }

    result
}

/// 指定された属性が付与されたフィールドを取得する。
///
/// ```
/// struct Foo {
///     #[getter(ret="val")]
///     a: i32,
///     #[setter(ret="ref")]}
///     b: i32,
///     c: i32,
/// ```
///
/// 上記のような構造体から、`getter`または`setter`属性が付与されたフィールドを取得する。
///
/// ```
/// let fields = retrieve_fields_by_names(&fields, &["getter", "setter"]);
/// ```
pub(crate) fn retrieve_fields_by_names<'a>(
    fields: &'a CommaPunctuatedFields,
    names: &[&str],
) -> Vec<&'a Field> {
    // 指定された属性が付与されたフィールドを格納するベクタ
    let mut result = vec![];
    // 構造体のフィールドを走査
    for field in fields {
        field.attrs.iter().for_each(|attr| {
            // 指定された属性が付与されたフィールドか確認
            for name in names {
                if attr.path().is_ident(name) {
                    result.push(field);
                }
            }
        });
    }

    result
}

pub(crate) fn expr_to_string(expr: Option<Expr>) -> Option<String> {
    let expr = expr.as_ref()?;
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(lit_str) => Some(lit_str.value()),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn expr_to_u8(expr: Expr) -> Result<u8, ()> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Int(lit_int) => Ok(lit_int.base10_parse::<u8>().map_err(|_| ())?),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

/// #[debug_info(name="debug_name", fmt="0x{:04X}")]
///                   ^^^^^^^^^^^^ <- この部分を取得
pub(crate) fn retrieve_attr_value(attrs: &[Attribute], name: &str) -> syn::Result<Option<Expr>> {
    for attr in attrs {
        let name_values: CommaPunctuatedNameValues = attr
            .parse_args_with(Punctuated::parse_terminated)
            .map_err(|e| e)?;
        for nv in name_values {
            let att_ident = nv.path.get_ident().unwrap();
            if *att_ident == name {
                return Ok(Some(nv.value));
            }
        }
    }

    Ok(None)
}

pub(crate) fn is_unit_struct(data: &syn::Data) -> bool {
    match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Unit => true,
            _ => false,
        },
        _ => false,
    }
}
