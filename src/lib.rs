extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate serde_derive_internals;
extern crate syn;
extern crate serde;

use serde_derive_internals::{ast, Ctxt};
use syn::DeriveInput;

mod derive_enum;
mod derive_struct;


#[cfg(feature = "bytes")]
extern crate serde_bytes;



#[proc_macro_derive(TypescriptDefinition)]
pub fn derive_typescript_definition(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // eprintln!(".........[input] {}", input);
    let input: DeriveInput = syn::parse(input).unwrap();

    let cx = Ctxt::new();
    let container = ast::Container::from_ast(&cx, &input);

    let typescript = match container.data {
        ast::Data::Enum(variants) => {
            derive_enum::derive_enum(variants, &container.attrs)
        }
        ast::Data::Struct(style, fields) => {
            derive_struct::derive_struct(style, fields, &container.attrs)
        }
    };

    let typescript_string = typescript.to_string();
    let typescript_ident = syn::Ident::from(format!("{}___typescript_definition", container.ident));
    let export_ident = syn::Ident::from(format!("TS_EXPORT_{}", container.ident.to_string().to_uppercase()));

    // eprintln!("....[typescript] {:?}", typescript_string);
    // eprintln!("........[schema] {:?}", inner_impl);
    // eprintln!();
    // eprintln!();
    // eprintln!();

    let mut expanded = quote!{

        #[wasm_bindgen(typescript_custom_section)]
        const #export_ident : &'static str = #typescript_string;

    };

    if cfg!(any(debug_assertions, feature = "test-export")) {
        expanded.append_all(quote!{
            fn #typescript_ident ( ) -> &'static str {
                #typescript_string
            }
        });
    }

    cx.check().unwrap();

    expanded.into()
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

fn derive_field<'a>(_variant_idx: usize, _field_idx: usize, field: &ast::Field<'a>) -> quote::Tokens {
    let field_name = field.attrs.name().serialize_name();
    let ty = type_to_ts(&field.ty);
    quote!{
        #field_name: #ty
    }
}

fn derive_element<'a>(_variant_idx: usize, _element_idx: usize, field: &ast::Field<'a>) -> quote::Tokens {
    let ty = type_to_ts(&field.ty);
    quote!{
        #ty
    }
}
