extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate serde_derive_internals;
extern crate syn;

use serde_derive_internals::{ast, attr, Ctxt};
use syn::{DeriveInput, GenericArgument, PathArguments};

#[proc_macro_derive(TypescriptDefinition)]
pub fn derive_typescript_definition(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let cx = Ctxt::new();
    let container =
        ast::Container::from_ast(&cx, &input, serde_derive_internals::Derive::Serialize).unwrap();

    let typescript = match container.data {
        ast::Data::Struct(style, fields) => derive_struct(style, fields, &container.attrs),
        ast::Data::Enum(variants) => derive_enum(variants, &container.attrs),
    };

    let export_ident = syn::Ident::new(
        &format!("TS_EXPORT_{}", container.ident.to_string().to_uppercase()),
        proc_macro2::Span::call_site(),
    );

    let ident = &container.ident;
    let typescript_string = typescript.to_string();
    let output = quote! {
        #[wasm_bindgen(typescript_custom_section)]
        const #export_ident: &'static str = #typescript_string;
    };

    println!("{:?}", output.to_string());
    cx.check().unwrap();

    output.into()
}

fn type_to_ts(ty: &syn::Type) -> proc_macro2::TokenStream {
    use syn::Type::*;
    match ty {
        Slice(ty) => {
            let ty = type_to_ts(&ty.elem);
            quote! { Array<#ty> }
        }
        Array(ty) => {
            let ty = type_to_ts(&ty.elem);
            quote! { Array<#ty> }
        }
        Path(inner) => {
            let result = &inner.path.segments.last().unwrap().ident;
            match result.to_string().as_ref() {
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
                | "i128" | "isize" => quote! { number },
                "String" | "&str" | "&'static str" => quote! { string },
                "bool" => quote! { boolean },
                "Option" => {
                    let generic_arg = match inner.path.segments.last().unwrap().arguments {
                        PathArguments::None | PathArguments::Parenthesized(_) => {
                            panic!("expected bracketed params on Option type")
                        }
                        PathArguments::AngleBracketed(ref params) => params.args.first().unwrap(),
                    };
                    let ty = match generic_arg {
                        GenericArgument::Type(ty) => ty,
                        _ => panic!("TODO: error handling"),
                    };
                    let ty = type_to_ts(ty);
                    quote! {
                        #ty?
                    }
                }
                _ => quote! { #result },
            }
        }
        Ptr(..) => quote! { any },
        Reference(..) => quote! { any },
        BareFn(..) => quote! { any },
        Never(..) => quote! { any },
        Tuple(..) => quote! { any },
        TraitObject(..) => quote! { any },
        ImplTrait(..) => quote! { any },
        Paren(..) => quote! { any },
        Group(..) => quote! { any },
        Infer(..) => quote! { any },
        Macro(..) => quote! { any },
        Verbatim(..) => quote! { any },
        _ => quote! { any },
    }
}

fn derive_struct(
    style: ast::Style,
    fields: Vec<ast::Field>,
    attr_container: &attr::Container,
) -> proc_macro2::TokenStream {
    fn derive_struct_newtype(
        fields: Vec<ast::Field>,
        _attr_container: &attr::Container,
    ) -> proc_macro2::TokenStream {
        derive_element(0, 0, &fields[0])
    }

    fn derive_struct_unit(_attr_container: &attr::Container) -> proc_macro2::TokenStream {
        quote! {
            {}
        }
    }

    fn derive_struct_named_fields(
        fields: Vec<ast::Field>,
        _attr_container: &attr::Container,
    ) -> proc_macro2::TokenStream {
        let fields = fields
            .into_iter()
            .enumerate()
            .map(|(field_idx, field)| derive_field(0, field_idx, &field));
        quote! {
            {#(#fields),*}
        }
    }

    fn derive_struct_tuple(
        fields: Vec<ast::Field>,
        _attr_container: &attr::Container,
    ) -> proc_macro2::TokenStream {
        let fields = fields.into_iter().map(|field| type_to_ts(field.ty));
        quote! {
            [#(#fields),*]
        }
    }

    let tokens = match style {
        ast::Style::Struct => derive_struct_named_fields(fields, attr_container),
        ast::Style::Newtype => derive_struct_newtype(fields, attr_container),
        ast::Style::Tuple => derive_struct_tuple(fields, attr_container),
        ast::Style::Unit => derive_struct_unit(attr_container),
    };

    tokens
}

fn derive_field(
    _variant_idx: usize,
    _field_idx: usize,
    field: &ast::Field,
) -> proc_macro2::TokenStream {
    let field_name = field.attrs.name().serialize_name();
    let ty = type_to_ts(&field.ty);
    quote! {
        #field_name: #ty
    }
}

fn derive_element(
    _variant_idx: usize,
    _element_idx: usize,
    field: &ast::Field,
) -> proc_macro2::TokenStream {
    let ty = type_to_ts(&field.ty);
    quote! {
        #ty
    }
}

fn derive_enum(
    variants: Vec<ast::Variant>,
    _attr_container: &attr::Container,
) -> proc_macro2::TokenStream {
    fn derive_unit_variant<'a>(variant_name: &str) -> proc_macro2::TokenStream {
        quote! {
            { "tag": #variant_name, }
        }
    }

    fn derive_newtype_variant(
        variant_name: &str,
        _variant_idx: usize,
        field: &ast::Field,
    ) -> proc_macro2::TokenStream {
        let ty = type_to_ts(&field.ty);
        quote! {
            { "tag": #variant_name, "fields": #ty, }
        }
    }

    fn derive_struct_variant<'a>(
        variant_name: &str,
        variant_idx: usize,
        fields: &Vec<ast::Field<'a>>,
    ) -> proc_macro2::TokenStream {
        let fields = fields
            .into_iter()
            .enumerate()
            .map(|(field_idx, field)| derive_field(variant_idx, field_idx, field));
        let contents = quote! {
            [#(#fields),*]
        };
        quote! {
            { "tag": #variant_name, "fields": #contents, }
        }
    }

    fn derive_tuple_variant<'a>(
        variant_name: &str,
        _variant_idx: usize,
        fields: &Vec<ast::Field<'a>>,
    ) -> proc_macro2::TokenStream {
        let fields = fields
            .into_iter()
            .enumerate()
            .map(|(element_idx, field)| derive_element(0, element_idx, &field));
        let contents = quote! {
            {#(#fields),*}
        };
        quote! {
            {"tag": #variant_name, "fields": #contents, }
        }
    }

    let variants = variants
        .into_iter()
        .enumerate()
        .map(|(variant_idx, variant)| {
            let variant_name = variant.attrs.name().serialize_name();
            match variant.style {
                ast::Style::Struct => {
                    derive_struct_variant(&variant_name, variant_idx, &variant.fields)
                }
                ast::Style::Newtype => {
                    derive_newtype_variant(&variant_name, variant_idx, &variant.fields[0])
                }
                ast::Style::Tuple => {
                    derive_tuple_variant(&variant_name, variant_idx, &variant.fields)
                }
                ast::Style::Unit => derive_unit_variant(&variant_name),
            }
        });

    quote! {
        #(#variants)|*
    }
}
