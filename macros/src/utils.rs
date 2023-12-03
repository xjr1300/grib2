use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed};

pub(crate) type FieldCollection = syn::punctuated::Punctuated<syn::Field, syn::token::Comma>;

/// 構造体のフィールドを取得する。
pub(crate) fn retrieve_struct_fields(input: DeriveInput) -> syn::Result<FieldCollection> {
    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => Ok(named),
        _ => Err(syn::Error::new_spanned(input, "expected struct")),
    }
}
