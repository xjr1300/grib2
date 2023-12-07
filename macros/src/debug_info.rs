use proc_macro2::{Ident, Literal, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    punctuated::Punctuated, Attribute, DeriveInput, Expr, Field, ImplGenerics, Lit, TypeGenerics,
    WhereClause,
};

use crate::utils::{
    expr_to_string, expr_to_u8, is_unit_struct, retrieve_fields_by_names, retrieve_struct_fields,
    retrieve_value_from_name_value, CommaPunctuatedNameValues,
};

pub(crate) fn derive_section_debug_info_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    // 構造体のジェネリックスを取得
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    // 構造体に付与されたsection属性を取得
    let section_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("section"))
        .ok_or_else(|| syn::Error::new_spanned(&input, "section attribute not found"))?;
    // section属性の値を取得
    let section_attr_values = retrieve_section_attr_values(section_attr)?;
    let section_number = Literal::u8_unsuffixed(section_attr_values.number);
    let section_name = Literal::string(&section_attr_values.name);

    let token_stream = match is_unit_struct(&input.data) {
        true => derive_section_unit_struct_impl(input.ident, section_number, section_name),
        false => derive_section_struct_impl(
            &input,
            impl_generics,
            ty_generics,
            where_clause,
            section_number,
            section_name,
        )?,
    };

    Ok(token_stream)
}

fn derive_section_unit_struct_impl(
    ident: Ident,
    section_number: Literal,
    section_name: Literal,
) -> TokenStream2 {
    quote! {
        impl #ident {
            pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
            where
                W: std::io::Write,
            {
                writeln!(writer, "第{}節:{}", #section_number, #section_name)?;

                Ok(())
            }
        }
    }
}

fn derive_section_struct_impl(
    input: &DeriveInput,
    impl_generics: ImplGenerics<'_>,
    ty_generics: TypeGenerics<'_>,
    where_clause: Option<&WhereClause>,
    section_number: Literal,
    section_name: Literal,
) -> syn::Result<TokenStream2> {
    // フィールドの識別子を取得
    let ident = &input.ident;
    // 構造体のフィールドを取得
    let fields = retrieve_struct_fields(input)?;
    // debug_infoまたはdebug_template属性が付与されたフィールドを取得
    let fields = retrieve_fields_by_names(&fields, &["debug_info", "debug_template"]);
    // debug_template属性が付与されたフィールドが存在するか確認
    let exists_debug_template = fields.iter().any(|field| {
        field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("debug_template"))
    });
    // フィールドごとにデバッグ情報を取得する構文木を生成
    let mut debug_infos = vec![];
    for field in fields.iter() {
        debug_infos.push(derive_debug_info_statement_impl(field)?);
    }

    let token_stream = if exists_debug_template {
        quote!(
            impl #impl_generics #ident #ty_generics #where_clause {
                pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
                where
                    T: DebugTemplate<W>,
                    W: std::io::Write,
                {
                    writeln!(writer, "第{}節:{}", #section_number, #section_name)?;
                    #(
                        #debug_infos
                    )*

                    Ok(())
                }
            }
        )
    } else {
        quote!(
            impl #ident {
                pub fn debug_info<W>(&self, writer: &mut W) -> std::io::Result<()>
                where
                    W: std::io::Write,
                {
                    writeln!(writer, "第{}節:{}", #section_number, #section_name)?;
                    #(
                        #debug_infos
                    )*

                    Ok(())
                }
            }
        )
    };

    Ok(token_stream)
}

fn derive_debug_info_statement_impl(field: &Field) -> syn::Result<TokenStream2> {
    // フィールドの識別子を取得
    let field_ident = field.ident.as_ref().unwrap();
    // フィールドがdebug_info属性を持つか確認
    let is_debug_info = field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("debug_info"));

    let token_stream = if is_debug_info {
        let name = retrieve_value_from_name_value(&field.attrs, "debug_info", "name").ok_or_else(
            || syn::Error::new_spanned(field, "name attribute not found in debug_info attribute"),
        )?;
        match retrieve_value_from_name_value(&field.attrs, "debug_info", "fmt") {
            Some(fmt) => {
                quote! {
                    writeln!(writer, "    {}: {}", #name, format!(#fmt, self.#field_ident))?;
                }
            }
            None => {
                quote! {
                    writeln!(writer, "    {}: {}", #name, self.#field_ident)?;
                }
            }
        }
    } else {
        quote! {
            self.#field_ident.debug_info(writer)?;
        }
    };

    Ok(token_stream)
}

struct SectionAttrValues {
    number: u8,
    name: String,
}

/// #[section(number=1, name="section_name")]
///           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- この部分を取得
fn retrieve_section_attr_values(attrs: &Attribute) -> syn::Result<SectionAttrValues> {
    let name_values: CommaPunctuatedNameValues = attrs
        .parse_args_with(Punctuated::parse_terminated)
        .map_err(|err| {
            syn::Error::new_spanned(attrs, format!("failed to parse getter attribute: {}", err))
        })?;
    let mut number: Option<Expr> = None;
    let mut name: Option<Expr> = None;
    for nv in name_values {
        let att_ident = nv.path.get_ident().unwrap();
        if *att_ident == "number" {
            number = Some(nv.value);
        } else if *att_ident == "name" {
            name = Some(nv.value);
        }
    }
    if number.is_none() || name.is_none() {
        return Err(syn::Error::new_spanned(
            attrs,
            "section attribute must have number and name",
        ));
    }

    // number属性の値をu8で取得
    let number = expr_to_u8(number.unwrap()).map_err(|_| {
        syn::Error::new_spanned(
            attrs,
            "section attribute's number argument must be u8 literal",
        )
    })?;
    // name属性の値を文字列で取得
    let name = expr_to_string(name).unwrap();

    Ok(SectionAttrValues { number, name })
}

pub(crate) fn derive_template_debug_info_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    // 構造体の識別子を取得
    let ident = &input.ident;
    // 構造体のフィールドを取得
    let fields = retrieve_struct_fields(&input)?;
    // debug_info属性が付与されたフィールドを取得
    let fields = retrieve_fields_by_names(&fields, &["debug_info"]);
    // フィールドごとにデバッグ情報を取得する文を生成
    let mut debug_infos = vec![];
    for field in fields.iter() {
        debug_infos.push(derive_template_debug_statement_impl(field)?);
    }

    Ok(quote! {
        impl<W> DebugTemplate<W> for #ident {
            fn debug_info(&self, writer: &mut W) -> std::io::Result<()>
            where
                W: std::io::Write,
            {
                #(
                    #debug_infos
                )*

                Ok(())
            }
        }
    })
}

fn derive_template_debug_statement_impl(field: &Field) -> syn::Result<TokenStream2> {
    // debug_info属性のdata_type属性を取得
    let data_type = retrieve_value_from_name_value(&field.attrs, "debug_info", "data_type");
    if data_type.is_some() {
        // ty属性が存在する場合
        let data_type = expr_to_string(data_type).unwrap();
        if data_type == "serial" {
            derive_template_debug_info_serial_statement_impl(field)
        } else {
            derive_template_debug_info_normal_statement_impl(field)
        }
    } else {
        // ty属性が存在しない場合
        derive_template_debug_info_normal_statement_impl(field)
    }
}

fn derive_template_debug_info_normal_statement_impl(field: &Field) -> syn::Result<TokenStream2> {
    // フィールドの識別子を取得
    let field_ident = field.ident.as_ref().unwrap();
    // debug_info属性のname属性を取得
    let name =
        retrieve_value_from_name_value(&field.attrs, "debug_info", "name").ok_or_else(|| {
            syn::Error::new_spanned(field, "name1 attribute not found in debug_info attribute")
        })?;
    // debug_info属性のfmt属性を取得
    match retrieve_value_from_name_value(&field.attrs, "debug_info", "fmt") {
        Some(fmt) => {
            // fmt属性が存在する場合
            Ok(quote! {
                writeln!(writer, "    {}: {}", #name, format!(#fmt, self.#field_ident))?;
            })
        }
        None => {
            // fmt属性が存在しない場合
            Ok(quote! {
                writeln!(writer, "    {}: {}", #name, self.#field_ident)?;
            })
        }
    }
}

fn derive_template_debug_info_serial_statement_impl(field: &Field) -> syn::Result<TokenStream2> {
    // フィールドの識別子を取得
    let field_ident = field.ident.as_ref().unwrap();
    // debug_info属性のname属性を取得
    let name =
        retrieve_value_from_name_value(&field.attrs, "debug_info", "name").ok_or_else(|| {
            syn::Error::new_spanned(field, "name2 attribute not found in debug_info attribute")
        })?;
    // debug_info属性のheader属性を取得
    let header =
        retrieve_value_from_name_value(&field.attrs, "debug_info", "header").ok_or_else(|| {
            syn::Error::new_spanned(field, "header attribute not found in debug_info attribute")
        })?;
    // debug_info属性のheader属性を取得
    let start = match retrieve_value_from_name_value(&field.attrs, "debug_info", "start") {
        Some(start) => {
            // start属性が存在する場合
            match &start {
                Expr::Lit(expr_lit) => match &expr_lit.lit {
                    Lit::Int(lit_int) => Ok(lit_int
                        .base10_parse::<i32>()
                        .map_err(|_| syn::Error::new_spanned(start, "start must be i32"))?),
                    _ => Err(syn::Error::new_spanned(start, "start must be i32")),
                },
                _ => Err(syn::Error::new_spanned(start, "start must be i32")),
            }
        }
        None => {
            // start属性が存在しない場合
            Ok(0)
        }
    }?;
    // debug_info属性のfmt属性を取得
    match retrieve_value_from_name_value(&field.attrs, "debug_info", "fmt") {
        Some(fmt) => {
            // fmt属性が存在する場合
            Ok(quote! {
                writeln!(writer, "    {}:", #name)?;
                for (i, value) in self.#field_ident.iter().enumerate() {
                    writeln!(writer, "        {}: {}", format!(#header, i as i32 + #start), format!(#fmt, value))?;
                }
            })
        }
        None => {
            // fmt属性が存在しない場合
            Ok(quote! {
                writeln!(writer, "    {}:", #name)?;
                for (i, value) in self.#field_ident.iter().enumerate() {
                    writeln!(writer, "        {}: {}", format!(#header, i as i32 + #start),  value)?;
                }
            })
        }
    }
}
