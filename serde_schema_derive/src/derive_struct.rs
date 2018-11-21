use quote;
use serde_derive_internals::{ast, attr};
use crate::types::TypeId;

use super::{derive_element, derive_field, derive_register_field_types, variant_field_type_variable};

pub fn derive_struct<'a>(
    style: ast::Style,
    fields: Vec<ast::Field<'a>>,
    attr_container: &attr::Container,
) -> ((), quote::Tokens) {
    ((), match style {
        ast::Style::Struct => derive_struct_named_fields(fields, attr_container).1,
        ast::Style::Newtype => derive_struct_newtype(fields, attr_container).1,
        ast::Style::Tuple => derive_struct_tuple(fields, attr_container).1,
        ast::Style::Unit => derive_struct_unit(attr_container).1,
    })
}


#[derive(Clone, Debug, PartialEq, Eq)]
enum MockTypeId {
    Boolean,
    String,
    Uint64,
    Int64,
    Unknown,
    Custom(usize),
}

impl TypeId for MockTypeId {
    const UNIT: MockTypeId = MockTypeId::Unknown;
    const BOOL: MockTypeId = MockTypeId::Boolean;
    const I8: MockTypeId = MockTypeId::Unknown;
    const I16: MockTypeId = MockTypeId::Unknown;
    const I32: MockTypeId = MockTypeId::Unknown;
    const I64: MockTypeId = MockTypeId::Int64;
    const U8: MockTypeId = MockTypeId::Unknown;
    const U16: MockTypeId = MockTypeId::Unknown;
    const U32: MockTypeId = MockTypeId::Unknown;
    const U64: MockTypeId = MockTypeId::Uint64;
    const F32: MockTypeId = MockTypeId::Unknown;
    const F64: MockTypeId = MockTypeId::Unknown;
    const CHAR: MockTypeId = MockTypeId::Unknown;
    const STR: MockTypeId = MockTypeId::String;
    const BYTES: MockTypeId = MockTypeId::Unknown;
}

fn derive_struct_newtype<'a>(
    fields: Vec<ast::Field<'a>>,
    attr_container: &attr::Container,
) -> ((), quote::Tokens) {
    let name = attr_container.name().serialize_name();
    let expanded_type_ids = derive_register_field_types(0, fields.iter());
    let (type_id_ident_RAW, type_id_ident) = variant_field_type_variable(0, 0);

    // let ty = MockTypeId::Custom(55);    
    // let builder = ::serde_schema::types::Type::<MockTypeId>::build()
    //     .newtype_struct_type(&name, type_id_ident_RAW);

    ((), quote!{
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema,
            ::serde_schema::types::Type::build()
                .newtype_struct_type(#name, #type_id_ident))
    })
}

fn derive_struct_unit(attr_container: &attr::Container) -> ((), quote::Tokens) {
    let name = attr_container.name().serialize_name();
    ((), quote!{
        ::serde_schema::Schema::register_type(schema,
            ::serde_schema::types::Type::build().unit_struct_type(#name))
    })
}

fn derive_struct_named_fields<'a>(
    fields: Vec<ast::Field<'a>>,
    attr_container: &attr::Container,
) -> ((), quote::Tokens) {
    let len = fields.len();
    let name = attr_container.name().serialize_name();

    let expanded_type_ids = derive_register_field_types(0, fields.iter());

    let mut expanded_build_type = quote!{
        ::serde_schema::types::Type::build()
            .struct_type(#name, #len)
    };
    for (field_idx, field) in fields.iter().enumerate() {
        expanded_build_type.append_all(derive_field(0, field_idx, field).1);
    }
    expanded_build_type.append_all(quote!{
        .end()
    });

    ((), quote!{
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema, #expanded_build_type)
    })
}

fn derive_struct_tuple<'a>(
    fields: Vec<ast::Field<'a>>,
    attr_container: &attr::Container,
) -> ((), quote::Tokens) {
    let len = fields.len();
    let name = attr_container.name().serialize_name();

    let expanded_type_ids = derive_register_field_types(0, fields.iter());

    let mut expanded_build_type = quote!{
        ::serde_schema::types::Type::build()
            .tuple_struct_type(#name, #len)
    };
    for (element_idx, _) in fields.iter().enumerate() {
        expanded_build_type.append_all(derive_element(0, element_idx).1);
    }
    expanded_build_type.append_all(quote!{
        .end()
    });

    ((), quote!{
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema, #expanded_build_type)
    })
}
