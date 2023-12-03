use std::str::FromStr;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Visibility};

use crate::utils::{
    retrieve_attr_fields, retrieve_getter_attr_values, retrieve_struct_fields, FieldAttrPair,
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
    let field_attrs = retrieve_attr_fields(&fields, "getter");
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
