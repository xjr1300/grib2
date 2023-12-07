use std::str::FromStr;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{punctuated::Punctuated, Attribute, DeriveInput, Expr, Visibility};

use crate::utils::{
    expr_to_string, retrieve_field_attrs_by_names, retrieve_struct_fields,
    CommaPunctuatedNameValues, FieldAttrPair,
};

pub(crate) fn derive_getter_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    // 構造体の識別子を取得
    let ident = &input.ident;
    // 構造体のジェネリックスを取得
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // 構造体のフィールドのゲッターメソッドを導出
    let getter_methods = derive_getter_methods(&input)?;

    // ゲッターメソッドを構造体に実装する構文木を生成
    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #getter_methods
        }
    })
}

fn derive_getter_methods(input: &DeriveInput) -> syn::Result<TokenStream2> {
    // 構造体の可視性を取得
    let vis = &input.vis;
    // 構造体のフィールドを取得
    let fields = retrieve_struct_fields(input)?;
    // getter属性が付与されたフィールドとgetter属性を取得
    let field_attrs = retrieve_field_attrs_by_names(&fields, &["getter"]);
    // 構造体のフィールドのゲッターメソッドの構文木を生成
    let mut getter_methods: Vec<TokenStream2> = vec![];
    for field_attr in field_attrs.iter() {
        getter_methods.push(derive_getter_method(vis, field_attr)?);
    }

    // 構造体の各フィールドのゲッターメソッドの構文木を結合
    Ok(quote! {
        #(
            #getter_methods
        )*
    })
}

fn derive_getter_method(vis: &Visibility, field_attr: &FieldAttrPair) -> syn::Result<TokenStream2> {
    // フィールドの識別子を取得
    let field_ty = &field_attr.field.ty;
    let field_ident = field_attr.field.ident.as_ref().unwrap();
    // getter属性のret属性とrty属性を取得
    let values = retrieve_getter_attr_values(field_attr)?;

    let token_stream = if values.ret == "val" {
        quote! {
            #vis fn #field_ident(&self) -> #field_ty {
                self.#field_ident
            }
        }
    } else if values.ret == "ref" && values.rty.is_none() {
        quote! {
            #vis fn #field_ident(&self) -> &#field_ty {
                &self.#field_ident
            }
        }
    } else {
        let rty = TokenStream2::from_str(&values.rty.unwrap()).unwrap();
        quote! {
            #vis fn #field_ident(&self) -> #rty {
                &self.#field_ident
            }
        }
    };

    Ok(token_stream)
}

pub(crate) fn derive_template_getter_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    let template_getter = input
        .attrs
        .iter()
        .find(|attr| attr.meta.path().is_ident("template_getter"))
        .ok_or_else(|| syn::Error::new_spanned(&input, "template_getter attribute not found"))?;

    let attr = retrieve_template_getter_values(template_getter)?;
    let section = TokenStream2::from_str(&attr.section)?;
    let member = TokenStream2::from_str(&attr.member)?;

    // テンプレート構造体の識別子を取得
    let template_ident = &input.ident;
    // 構造体のジェネリックスを取得
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    // テンプレート構造体の可視性を取得
    let vis = &input.vis;

    // 構造体のフィールドを取得
    let fields = retrieve_struct_fields(&input)?;
    // getter属性が付与されたフィールドとgetter属性を取得
    let field_attrs = retrieve_field_attrs_by_names(&fields, &["getter"]);
    // 構造体のフィールドのゲッターメソッドの構文木を生成
    let mut getter_methods: Vec<TokenStream2> = vec![];
    for field_attr in field_attrs.iter() {
        getter_methods.push(derive_template_getter_method(vis, &member, field_attr)?);
    }

    Ok(quote! {
        impl #impl_generics #section<#template_ident> #ty_generics #where_clause {
            #(
                #getter_methods
            )*
        }
    })
}

fn derive_template_getter_method(
    vis: &Visibility,
    member: &TokenStream2,
    field_attr: &FieldAttrPair,
) -> syn::Result<TokenStream2> {
    // フィールドの識別子を取得
    let field_ty = &field_attr.field.ty;
    let field_ident = field_attr.field.ident.as_ref().unwrap();
    // getter属性のret属性とrty属性を取得
    let values = retrieve_getter_attr_values(field_attr)?;

    let token_stream = if values.ret == "val" {
        quote! {
            #vis fn #field_ident(&self) -> #field_ty {
                self.#member.#field_ident
            }
        }
    } else if values.ret == "ref" && values.rty.is_none() {
        quote! {
            #vis fn #field_ident(&self) -> &#field_ty {
                &self.#member.#field_ident
            }
        }
    } else {
        let rty = TokenStream2::from_str(&values.rty.unwrap()).unwrap();
        quote! {
            #vis fn #field_ident(&self) -> #rty {
                &self.#member.#field_ident
            }
        }
    };

    Ok(token_stream)
}

struct TemplateGetterAttrValues {
    /// template_getter属性のsection属性の値
    section: String,
    /// template_getter属性のmember属性の値
    member: String,
}

/// #[template_getter(section="Section2", member="template2")]
///                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- この部分を取得
fn retrieve_template_getter_values(attr: &Attribute) -> syn::Result<TemplateGetterAttrValues> {
    let name_values: CommaPunctuatedNameValues = attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err(|err| {
            syn::Error::new_spanned(attr, format!("failed to parse getter attribute: {}", err))
        })?;
    // getterのret属性とrty属性を取得
    let mut section: Option<Expr> = None;
    let mut member: Option<Expr> = None;
    for nv in name_values {
        let att_ident = nv.path.get_ident().unwrap();
        if *att_ident == "section" {
            section = Some(nv.value);
        } else if *att_ident == "member" {
            member = Some(nv.value);
        }
    }
    if section.is_none() || member.is_none() {
        return Err(syn::Error::new_spanned(
            attr,
            "template_getter attribute must have `section` and `member` argument",
        ));
    }

    // section属性の値を文字列で取得
    let section = expr_to_string(section).unwrap();
    // member属性の値を文字列で取得
    let member = expr_to_string(member).unwrap();

    Ok(TemplateGetterAttrValues { section, member })
}

struct GetterAttrValues {
    /// getter属性のret属性の値
    pub ret: String,
    /// getter属性のrty属性の値
    pub rty: Option<String>,
}

// getter属性のカンマで区切られた属性を取得
// getter(ret = "ref", rty = "&str")
//        ^^^^^^^^^^^^^^^^^^^^^^^^^  <- この部分を取得
fn retrieve_getter_attr_values(field_attr: &FieldAttrPair) -> syn::Result<GetterAttrValues> {
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
