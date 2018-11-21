// TODO delete
#![allow(non_snake_case)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate serde_derive_internals;
extern crate syn;
extern crate serde;

use std::borrow::Borrow;

use serde_derive_internals::{ast, Ctxt};
use syn::DeriveInput;

mod derive_enum;
mod derive_struct;


#[cfg(feature = "bytes")]
extern crate serde_bytes;



#[proc_macro_derive(SchemaSerialize)]
pub fn derive_schema_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // eprintln!(".........[input] {}", input);
    let input: DeriveInput = syn::parse(input).unwrap();

    let cx = Ctxt::new();
    let container = ast::Container::from_ast(&cx, &input);

    let (typescript, _) = match container.data {
        ast::Data::Enum(variants) => {
            derive_enum::derive_enum(variants, &container.attrs)
        }
        ast::Data::Struct(style, fields) => {
            derive_struct::derive_struct(style, fields, &container.attrs)
        }
    };

    let typescript_string = typescript.to_string();
    let typescript_ident = syn::Ident::from(format!("{}_typescript_definition", container.ident));

    // eprintln!("....[typescript] {:?}", typescript_string);
    // eprintln!("........[schema] {:?}", inner_impl);
    // eprintln!();
    // eprintln!();
    // eprintln!();

    let expanded = quote!{
        fn #typescript_ident ( ) -> &'static str {
            #typescript_string
        }
    };

    cx.check().unwrap();

    expanded.into()
}

fn variant_field_type_variable(variant_idx: usize, field_idx: usize) -> (String, syn::Ident) {
    let var = format!("type_id_{}_{}", variant_idx, field_idx);
    (var.clone(), syn::Ident::from(var))
}

fn collapse_list_bracket(body: Vec<quote::Tokens>) -> quote::Tokens {
    if body.len() == 1 {
        body[0].clone()
    } else {
        let tokens = body.into_iter().fold(quote!{}, |mut agg, tokens| { agg.append_all(quote!{ #tokens , }); agg });
        quote!{ [ #tokens ] }
    }
}

fn collapse_list_brace(body: Vec<quote::Tokens>) -> quote::Tokens {
    let tokens = body.into_iter().fold(quote!{}, |mut agg, tokens| { agg.append_all(quote!{ #tokens , }); agg });
    quote!{ { #tokens } }
}

fn type_to_ts(ty: &syn::Type) -> quote::Tokens {
    // println!("??? {:?}", ty);
    use syn::Type::*;
    match ty {
        Slice(..) => quote!{ any },
        Array(..) => quote!{ any },
        Ptr(..) => quote!{ any },
        Reference(..) => quote!{ any },
        BareFn(..) => quote!{ any },
        Never(..) => quote!{ any },
        Tuple(..) => quote!{ any },
        Path(inner) => {
            // let ty_string = format!("{}", inner.path);
            let result = quote!{ #inner };
            match result.to_string().as_ref() {
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" =>
                    quote! { number },
                "String" | "&str" | "&'static str" =>
                    quote! { string },
                "bool" => quote!{ boolean },
                _ => quote! { any },
            }
        }
        TraitObject(..) => quote!{ any },
        ImplTrait(..) => quote!{ any },
        Paren(..) => quote!{ any },
        Group(..) => quote!{ any },
        Infer(..) => quote!{ any },
        Macro(..) => quote!{ any },
        Verbatim(..) => quote!{ any },
    }
}

fn derive_register_field_types<'a, I>(variant_idx: usize, fields: I) -> (quote::Tokens, quote::Tokens)
where
    I: IntoIterator,
    I::Item: Borrow<ast::Field<'a>>,
{
    let mut expanded = quote!{};
    let mut expanded_TS = vec![];
    for (field_idx, field_item) in fields.into_iter().enumerate() {
        let field = field_item.borrow();
        let field_type = &field.ty;
        let (type_id_ident_TS, type_id_ident) = variant_field_type_variable(variant_idx, field_idx);
        expanded.append_all(quote!{
            let #type_id_ident =
                <#field_type as ::serde_schema::SchemaSerialize>::schema_register(schema)?;
        });
        expanded_TS.push(type_to_ts(field_type));
    }
    (collapse_list_brace(expanded_TS), expanded)
}

fn derive_field<'a>(variant_idx: usize, field_idx: usize, field: &ast::Field<'a>) -> (quote::Tokens, quote::Tokens) {
    let (_, type_id_ident) = variant_field_type_variable(variant_idx, field_idx);
    let field_name = field.attrs.name().serialize_name();

    let ty = type_to_ts(&field.ty);
    (quote!{
        #field_name: #ty
    }, quote!{})
}

fn derive_element<'a>(variant_idx: usize, element_idx: usize, field: &ast::Field<'a>) -> (quote::Tokens, quote::Tokens) {
    let (type_id_ident_TS, type_id_ident) = variant_field_type_variable(variant_idx, element_idx);
    let ty = type_to_ts(&field.ty);
    (quote!{
        #ty
    }, quote!{})
}
