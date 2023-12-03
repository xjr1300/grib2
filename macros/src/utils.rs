use syn::punctuated::Punctuated;
use syn::Token;
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Expr, Field, Fields, FieldsNamed, Lit, MetaNameValue,
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

/// 指定された属性が付与されたフィールドを取得する。
pub(crate) fn retrieve_attr_fields<'a>(
    fields: &'a CommaPunctuatedFields,
    name: &str,
) -> Vec<FieldAttrPair<'a>> {
    // 指定された属性が付与されたフィールドとgetter属性を格納するベクタ
    let mut result = vec![];
    // 構造体のフィールドを走査
    for field in fields {
        field.attrs.iter().for_each(|attr| {
            // 指定された属性が付与されたフィールドか確認
            if attr.meta.path().is_ident(name) {
                result.push(FieldAttrPair { field, attr });
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

pub(crate) struct GetterAttrValues {
    /// getter属性のret属性の値
    pub ret: String,
    /// getter属性のrty属性の値
    pub rty: Option<String>,
}

// getter属性のカンマで区切られた属性を取得
// getter(ret = "ref", rty = "&str")
//        ^^^^^^^^^^^^^^^^^^^^^^^^^  <- この部分を取得
pub(crate) fn retrieve_getter_attr_values(
    field_attr: &FieldAttrPair,
) -> syn::Result<GetterAttrValues> {
    let name_values: CommaPunctuatedNameValues = field_attr
        .attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err(|err| {
            syn::Error::new_spanned(
                field_attr.attr,
                format!("failed to parse getter attribute: {}", err),
            )
        })?;

    // getterのret属性とrty属性を取得
    let mut ret: Option<Expr> = None;
    let mut rty: Option<Expr> = None;
    for nv in name_values {
        let att_ident = nv.path.get_ident().unwrap();
        if *att_ident == "ret" {
            ret = Some(nv.value);
        } else if *att_ident == "rty" {
            rty = Some(nv.value);
        }
    }

    // ret属性が指定されていない場合はエラー
    if ret.is_none() {
        return Err(syn::Error::new_spanned(
            field_attr.field,
            "getter attribute must have `ret` argument",
        ));
    }

    // ret属性の値を文字列で取得
    let ret = expr_to_string(ret).unwrap();
    // rty属性の値を文字列で取得
    let rty = expr_to_string(rty);

    // ret属性がvalの場合にrty属性が指定されていたらエラー
    if ret == "val" && rty.is_some() {
        return Err(syn::Error::new_spanned(
            field_attr.field,
            "getter attribute `rty` argument must not be specified when `ret` argument is `val`",
        ));
    }

    Ok(GetterAttrValues { ret, rty })
}
