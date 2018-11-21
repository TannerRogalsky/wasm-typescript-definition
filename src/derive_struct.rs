use quote;
use serde_derive_internals::{ast, attr};
use type_to_ts;
use collapse_list_bracket;
use collapse_list_brace;
use super::{derive_element, derive_field};

pub fn derive_struct<'a>(
    style: ast::Style,
    fields: Vec<ast::Field<'a>>,
    attr_container: &attr::Container,
) -> quote::Tokens {
    let tokens = match style {
        ast::Style::Struct => derive_struct_named_fields(fields, attr_container),
        ast::Style::Newtype => derive_struct_newtype(fields, attr_container),
        ast::Style::Tuple => derive_struct_tuple(fields, attr_container),
        ast::Style::Unit => derive_struct_unit(attr_container),
    };

    tokens
}

fn derive_struct_newtype<'a>(
    fields: Vec<ast::Field<'a>>,
    _attr_container: &attr::Container,
) -> quote::Tokens {
    derive_element(0, 0, &fields[0])
}

fn derive_struct_unit(_attr_container: &attr::Container) -> quote::Tokens {
    quote!{
        {}
    }
}

fn derive_struct_named_fields<'a>(
    fields: Vec<ast::Field<'a>>,
    _attr_container: &attr::Container,
) -> quote::Tokens {
    collapse_list_brace(fields.into_iter().enumerate()
        .map(|(field_idx, field)| derive_field(0, field_idx, &field))
        .collect::<Vec<_>>())
}

fn derive_struct_tuple<'a>(
    fields: Vec<ast::Field<'a>>,
    _attr_container: &attr::Container,
) -> quote::Tokens {
    collapse_list_bracket(fields.into_iter()
        .map(|field| type_to_ts(field.ty))
        .collect::<Vec<_>>())
}
