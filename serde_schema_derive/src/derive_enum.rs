use quote;
use serde_derive_internals::{ast, attr};
use collapse_list_bracket;
use collapse_list_brace;
use type_to_ts;
use super::{derive_element, derive_field, derive_register_field_types, variant_field_type_variable};

pub fn derive_enum<'a>(
    variants: Vec<ast::Variant<'a>>,
    attr_container: &attr::Container,
) -> (quote::Tokens, quote::Tokens) {
    let name = attr_container.name().serialize_name();
    let len = variants.len();

    let mut expanded_type_ids = quote!{};
    for (variant_idx, variant) in variants.iter().enumerate() {
        expanded_type_ids.append_all(derive_register_field_types(
            variant_idx,
            variant.fields.iter(),
        ).1);
    }

    let mut expanded_build_type = quote!{
        ::serde_schema::types::Type::build()
            .enum_type(#name, #len)
    };

    for (variant_idx, variant) in variants.iter().enumerate() {
        let variant_name = variant.attrs.name().serialize_name();
        let expanded_build_variant = match variant.style {
            ast::Style::Struct => {
                derive_struct_variant(&variant_name, variant_idx, &variant.fields)
            }
            ast::Style::Newtype => derive_newtype_variant(&variant_name, variant_idx, &variant.fields[0]),
            ast::Style::Tuple => derive_tuple_variant(&variant_name, variant_idx, &variant.fields),
            ast::Style::Unit => derive_unit_variant(&variant_name),
        }.1;
        expanded_build_type.append_all(expanded_build_variant);
    }

    expanded_build_type.append_all(quote!{
        .end()
    });


    let expanded_build_type_TS = variants.into_iter().enumerate()
        .map(|(variant_idx, variant)| {
            let variant_name = variant.attrs.name().serialize_name();
            match variant.style {
                ast::Style::Struct => {
                    derive_struct_variant(&variant_name, variant_idx, &variant.fields)
                }
                ast::Style::Newtype => derive_newtype_variant(&variant_name, variant_idx, &variant.fields[0]),
                ast::Style::Tuple => derive_tuple_variant(&variant_name, variant_idx, &variant.fields),
                ast::Style::Unit => derive_unit_variant(&variant_name),
            }.0
        })
        .fold(quote!{}, |mut agg, tokens| { agg.append_all(tokens); agg });

    (expanded_build_type_TS, quote!{
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema, #expanded_build_type)
    })
}

fn derive_unit_variant<'a>(variant_name: &str) -> (quote::Tokens, quote::Tokens) {
    (quote!{
        | { "tag": #variant_name, }
    }, quote!{
        .unit_variant(#variant_name)
    })
}

fn derive_newtype_variant<'a>(variant_name: &str, variant_idx: usize, 
    field: &ast::Field<'a>) -> (quote::Tokens, quote::Tokens) {
    let (_, field_type) = variant_field_type_variable(variant_idx, 0);
    let ty = type_to_ts(&field.ty); 
    (quote!{
        | { "tag": #variant_name, "fields": #ty, }
    }, quote!{
        .newtype_variant(#variant_name, #field_type)
    })
}

fn derive_struct_variant<'a>(
    variant_name: &str,
    variant_idx: usize,
    fields: &Vec<ast::Field<'a>>,
) -> (quote::Tokens, quote::Tokens) {
    let fields_len = fields.len();
    let mut expanded = quote!{
        .struct_variant(#variant_name, #fields_len)
    };
    for (field_idx, field) in fields.iter().enumerate() {
        expanded.append_all(derive_field(variant_idx, field_idx, field).1);
    }
    expanded.append_all(quote!{
        .end()
    });

    let contents = collapse_list_brace(fields.into_iter().enumerate()
        .map(|(field_idx, field)| derive_field(variant_idx, field_idx, field).0)
        .collect::<Vec<_>>());
    let expanded_TS = quote!{
        | { "tag": #variant_name, "fields": #contents, }
    };

    (expanded_TS, expanded)
}

fn derive_tuple_variant<'a>(
    variant_name: &str,
    variant_idx: usize,
    fields: &Vec<ast::Field<'a>>,
) -> (quote::Tokens, quote::Tokens) {
    let fields_len = fields.len();
    let mut expanded = quote!{
        .tuple_variant(#variant_name, #fields_len)
    };
    for (field_idx, field) in fields.iter().enumerate() {
        expanded.append_all(derive_element(variant_idx, field_idx, field).1);
    }
    expanded.append_all(quote!{
        .end()
    });

    let contents = collapse_list_bracket(fields.into_iter().enumerate()
        .map(|(element_idx, field)| derive_element(0, element_idx, &field).0)
        .collect::<Vec<_>>());
    let expanded_TS = quote!{
        | {"tag": #variant_name, "fields": #contents, }
    };

    (expanded_TS, expanded)
}
