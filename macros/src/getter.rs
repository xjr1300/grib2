use std::str::FromStr;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Attribute, DeriveInput, Expr, Field, Lit, MetaNameValue, Token, Visibility};

use crate::utils::FieldCollection;

use super::utils::retrieve_struct_fields;

/// ```rust
/// #[derive(Getter)]
/// pub struct Foo {
///     #[getter(ret="val")]
///     a: i32,
///     #[getter(ret="ref")]
///     b: PathBuf,
///     #[getter(ret="ref", ty="&str")]
///     c: String,
/// }
/// ```
///
/// 上記構造体から次を導出する。
///
/// ```rust
/// impl Foo {
///     pub fn a(&self) -> i32 {
///         self.a
///     }
///     pub fn b(&self) -> &PathBuf {
///          &self.b
///     }
///     pub fn c(&self) -> &str {
///        &self.c
///     }
/// }
/// ```
pub(crate) fn derive_getter_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    // 構造体の識別子を取得
    let ident = input.clone().ident;
    // 構造体のジェネリックスを取得
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // 構造体のフィールドのゲッターメソッドを導出
    let getter_methods = derive_getter_methods(input.clone())?;

    // ゲッターメソッドを構造体に実装する構文木を生成
    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #getter_methods
        }
    })
}

struct FieldAttrPair<'a> {
    /// フィールド
    field: &'a Field,
    /// 属性
    attr: &'a Attribute,
}

fn derive_getter_methods(input: DeriveInput) -> syn::Result<TokenStream2> {
    // 構造体の可視性を取得
    let vis = input.clone().vis;
    // 構造体のフィールドを取得
    let fields = retrieve_struct_fields(input)?;
    // getter属性が付与されたフィールドとgetter属性を取得
    let fields = retrieve_fields_has_getter_attr(&fields);
    // 構造体のフィールドのゲッターメソッドの構文木を生成
    let mut getter_methods: Vec<TokenStream2> = vec![];
    for field in fields.iter() {
        getter_methods.push(derive_getter_method(field, &vis)?);
    }

    // 構造体の各フィールドのゲッターメソッドの構文木を結合
    Ok(quote! {
        #(
            #getter_methods
        )*
    })
}

fn retrieve_fields_has_getter_attr(fields: &FieldCollection) -> Vec<FieldAttrPair> {
    // getter属性が付与されたフィールドとgetter属性を格納するベクタ
    let mut result = vec![];
    // 構造体のフィールドを走査
    for field in fields {
        field.attrs.iter().for_each(|attr| {
            // getter属性が付与されたフィールドか確認
            if attr.meta.path().is_ident("getter") {
                result.push(FieldAttrPair { field, attr });
            }
        });
    }

    result
}

fn derive_getter_method(field_attr: &FieldAttrPair, vis: &Visibility) -> syn::Result<TokenStream2> {
    // フィールドの識別子を取得
    let field_ident = field_attr.field.ident.as_ref().unwrap();
    // フィールドの型を取得
    let field_ty = &field_attr.field.ty;
    // getter属性の引数をカンマ区切りで取得
    let name_values: Result<Punctuated<MetaNameValue, Token![,]>, syn::Error> = field_attr
        .attr
        .parse_args_with(Punctuated::parse_terminated);
    let name_values = name_values?;
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
    let ret = ret.unwrap();
    // ret属性の値がvalかrefでない場合はエラー
    let ret_val = match ret {
        Expr::Lit(expr_lit) => match expr_lit.lit {
            Lit::Str(lit_str) => {
                let ret = lit_str.value();
                if ret != "val" && ret != "ref" {
                    return Err(syn::Error::new_spanned(
                        field_attr.field,
                        "getter attribute `ret` argument must be `val` or `ref`",
                    ));
                }
                ret
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    field_attr.field,
                    "getter attribute `ret` argument must be `val` or `ref`",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                field_attr.field,
                "getter attribute `ret` argument must be `val` or `ref`",
            ));
        }
    };

    // rty属性が指定されていて、ret属性の値がvalの場合はエラー
    if rty.is_some() && ret_val == "val" {
        return Err(syn::Error::new_spanned(
            field_attr.field,
            "getter attribute `rty` argument cannot be specified when `ret` argument is `val`",
        ));
    }

    let token_stream = if ret_val == "val" {
        quote! {
            #vis fn #field_ident(&self) -> #field_ty {
                self.#field_ident
            }
        }
    } else if ret_val == "ref" && rty.is_none() {
        quote! {
            #vis fn #field_ident(&self) -> &#field_ty {
                &self.#field_ident
            }
        }
    } else {
        let rty = rty.unwrap();
        let r_type = match rty {
            Expr::Lit(expr_lit) => match expr_lit.lit {
                Lit::Str(lit_str) => TokenStream2::from_str(&lit_str.value()).unwrap(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        field_attr.field,
                        "getter attribute `rty` argument must be `ref`",
                    ));
                }
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    field_attr.field,
                    "getter attribute `rty` argument must be type",
                ));
            }
        };

        quote! {
            #vis fn #field_ident(&self) -> #r_type {
                &self.#field_ident
            }
        }
    };

    Ok(token_stream)
}
