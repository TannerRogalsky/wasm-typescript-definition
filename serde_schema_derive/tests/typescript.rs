extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_schema;
#[macro_use]
extern crate serde_schema_derive;
#[macro_use]
extern crate quote;

use std::borrow::Cow;
use quote::ToTokens;
use quote::Tokens;
use serde::de::value::Error;
use serde_schema::types::{Type, TypeId};
use serde_schema::{Schema, SchemaSerialize};

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

struct MockSchema(Vec<Type<MockTypeId>>);

impl Schema for MockSchema {
    type TypeId = MockTypeId;
    type Error = Error;

    fn register_type(&mut self, ty: Type<MockTypeId>) -> Result<MockTypeId, Error> {
        self.0.push(ty);
        Ok(MockTypeId::Custom(self.0.len() - 1))
    }
}

#[test]
fn unit_struct() {
    #[derive(Serialize, SchemaSerialize)]
    struct Unit;

    let mut schema = MockSchema(Vec::new());
    let type_id = Unit::schema_register(&mut schema).unwrap();

    assert_eq!(Unit_typescript_definition(), quote!{
        {}
    }.to_string());

    assert_eq!(type_id, MockTypeId::Custom(0));
    assert_eq!(schema.0.len(), 1);
    let a = &schema.0[0];
    assert_eq!(*a, Type::build().unit_struct_type("Unit"));
}

#[test]
fn newtype_struct() {
    #[derive(Serialize, SchemaSerialize)]
    struct Newtype(i64);

    let mut schema = MockSchema(Vec::new());
    let type_id = Newtype::schema_register(&mut schema).unwrap();

    assert_eq!(Newtype_typescript_definition(), quote!{
        number
    }.to_string());

    assert_eq!(type_id, MockTypeId::Custom(0));
    assert_eq!(schema.0.len(), 1);
    assert_eq!(
        schema.0[0],
        Type::build().newtype_struct_type("Newtype", MockTypeId::Int64)
    );
}

#[test]
fn tuple_struct() {
    #[derive(Serialize, SchemaSerialize)]
    struct Tuple(i64, String);

    let mut schema = MockSchema(Vec::new());
    let type_id = Tuple::schema_register(&mut schema).unwrap();

    assert_eq!(Tuple_typescript_definition(), quote!{
        [number, string,]
    }.to_string());

    assert_eq!(type_id, MockTypeId::Custom(0));
    assert_eq!(schema.0.len(), 1);
    assert_eq!(
        schema.0[0],
        Type::build()
            .tuple_struct_type("Tuple", 2)
            .element(MockTypeId::Int64)
            .element(MockTypeId::String)
            .end()
    );
}

#[test]
fn struct_with_borrowed_fields() {
    #[derive(Serialize, SchemaSerialize)]
    struct Borrow<'a> {
        raw: &'a str,
        cow: Cow<'a, str>
    }

    let mut schema = MockSchema(Vec::new());
    let type_id = Borrow::schema_register(&mut schema).unwrap();

    // TODO raw should be string(!)
    assert_eq!(Borrow_typescript_definition(), quote!{
        {"raw": any, "cow": any,}
    }.to_string());

    assert_eq!(type_id, MockTypeId::Custom(0));
    assert_eq!(schema.0.len(), 1);
    assert_eq!(
        schema.0[0],
        Type::build()
            .struct_type("Borrow", 2)
            .field("raw", MockTypeId::String)
            .field("cow", MockTypeId::String)
            .end()
    );
}

#[test]
fn struct_point_with_field_rename() {
    #[derive(Serialize, SchemaSerialize)]
    struct Point {
        #[serde(rename = "X")]
        x: i64,
        #[serde(rename = "Y")]
        y: i64,
    }

    let mut schema = MockSchema(Vec::new());
    let type_id = Point::schema_register(&mut schema).unwrap();

    // TODO raw should be string(!)
    assert_eq!(Point_typescript_definition(), quote!{
        {"X": number, "Y": number,}
    }.to_string());

    assert_eq!(type_id, MockTypeId::Custom(0));
    assert_eq!(schema.0.len(), 1);
    assert_eq!(
        schema.0[0],
        Type::build()
            .struct_type("Point", 2)
            .field("X", MockTypeId::Int64)
            .field("Y", MockTypeId::Int64)
            .end()
    );
}

#[test]
fn enum_with_renamed_newtype_variants() {
    #[derive(Serialize, SchemaSerialize)]
    enum Enum {
        #[serde(rename = "Var1")]
        #[allow(unused)]
        V1(bool),
        #[serde(rename = "Var2")]
        #[allow(unused)]
        V2(i64),
        #[serde(rename = "Var3")]
        #[allow(unused)]
        V3(String),
    }

    let mut schema = MockSchema(Vec::new());
    let type_id = Enum::schema_register(&mut schema).unwrap();

    // TODO raw should be string(!)
    assert_eq!(Enum_typescript_definition(), quote!{
        | {"tag": "Var1", "fields": boolean,}
        | {"tag": "Var2", "fields": number,}
        | {"tag": "Var3", "fields": string,}
    }.to_string());

    assert_eq!(type_id, MockTypeId::Custom(0));
    assert_eq!(schema.0.len(), 1);
    assert_eq!(
        schema.0[0],
        Type::build()
            .enum_type("Enum", 3)
            .newtype_variant("Var1", MockTypeId::Boolean)
            .newtype_variant("Var2", MockTypeId::Int64)
            .newtype_variant("Var3", MockTypeId::String)
            .end()
    );
}

#[test]
fn enum_with_unit_variants() {
    #[derive(Serialize, SchemaSerialize)]
    enum Enum {
        #[allow(unused)]
        V1,
        #[allow(unused)]
        V2,
        #[allow(unused)]
        V3,
    }

    let mut schema = MockSchema(Vec::new());
    let type_id = Enum::schema_register(&mut schema).unwrap();

    assert_eq!(Enum_typescript_definition(), quote!{
        | {"tag": "V1",}
        | {"tag": "V2",}
        | {"tag": "V3",}
    }.to_string());

    assert_eq!(type_id, MockTypeId::Custom(0));
    assert_eq!(schema.0.len(), 1);
    assert_eq!(
        schema.0[0],
        Type::build()
            .enum_type("Enum", 3)
            .unit_variant("V1")
            .unit_variant("V2")
            .unit_variant("V3")
            .end()
    );
}

#[test]
fn enum_with_tuple_variants() {
    #[derive(Serialize, SchemaSerialize)]
    enum Enum {
        #[allow(unused)]
        V1(i64, String),
        #[allow(unused)]
        V2(i64, bool),
        #[allow(unused)]
        V3(i64, u64),
    }

    let mut schema = MockSchema(Vec::new());
    let type_id = Enum::schema_register(&mut schema).unwrap();

    // TODO raw should be string(!)
    assert_eq!(Enum_typescript_definition(), quote!{
        | {"tag": "V1", "fields": [number, string,],}
        | {"tag": "V2", "fields": [number, boolean,],}
        | {"tag": "V3", "fields": [number, number,],}
    }.to_string());

    assert_eq!(type_id, MockTypeId::Custom(0));
    assert_eq!(schema.0.len(), 1);
    assert_eq!(
        schema.0[0],
        Type::build()
            .enum_type("Enum", 3)
            .tuple_variant("V1", 2)
            .element(MockTypeId::Int64)
            .element(MockTypeId::String)
            .end()
            .tuple_variant("V2", 2)
            .element(MockTypeId::Int64)
            .element(MockTypeId::Boolean)
            .end()
            .tuple_variant("V3", 2)
            .element(MockTypeId::Int64)
            .element(MockTypeId::Uint64)
            .end()
            .end()
    );
}

#[test]
fn enum_with_struct_variants_and_renamed_fields() {
    #[derive(Serialize, SchemaSerialize)]
    enum Enum {
        #[allow(unused)]
        V1 {
            #[serde(rename = "Foo")]
            foo: bool,
        },
        #[allow(unused)]
        V2 {
            #[serde(rename = "Bar")]
            bar: i64,
            #[serde(rename = "Baz")]
            baz: u64,
        },
        #[allow(unused)]
        V3 {
            #[serde(rename = "Quux")]
            quux: String,
        },
    }

    let mut schema = MockSchema(Vec::new());
    let type_id = Enum::schema_register(&mut schema).unwrap();

    // TODO raw should be string(!)
    assert_eq!(Enum_typescript_definition(), quote!{
        | {"tag": "V1", "fields": { "Foo": boolean, }, }
        | {"tag": "V2", "fields": { "Bar": number, "Baz": number, }, }
        | {"tag": "V3", "fields": { "Quux": string, }, }
    }.to_string());

    assert_eq!(type_id, MockTypeId::Custom(0));
    assert_eq!(schema.0.len(), 1);
    assert_eq!(
        schema.0[0],
        Type::build()
            .enum_type("Enum", 2)
            .struct_variant("V1", 1)
            .field("Foo", MockTypeId::Boolean)
            .end()
            .struct_variant("V2", 2)
            .field("Bar", MockTypeId::Int64)
            .field("Baz", MockTypeId::Uint64)
            .end()
            .struct_variant("V3", 1)
            .field("Quux", MockTypeId::String)
            .end()
            .end()
    );
}