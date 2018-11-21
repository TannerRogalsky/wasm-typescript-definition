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

#[test]
fn unit_struct() {
    #[derive(Serialize, SchemaSerialize)]
    struct Unit;

    assert_eq!(Unit_typescript_definition(), quote!{
        {}
    }.to_string());
}

#[test]
fn newtype_struct() {
    #[derive(Serialize, SchemaSerialize)]
    struct Newtype(i64);

    assert_eq!(Newtype_typescript_definition(), quote!{
        number
    }.to_string());
}

#[test]
fn tuple_struct() {
    #[derive(Serialize, SchemaSerialize)]
    struct Tuple(i64, String);

    assert_eq!(Tuple_typescript_definition(), quote!{
        [number, string,]
    }.to_string());
}

#[test]
fn struct_with_borrowed_fields() {
    #[derive(Serialize, SchemaSerialize)]
    struct Borrow<'a> {
        raw: &'a str,
        cow: Cow<'a, str>
    }

    // TODO raw should be string(!)
    assert_eq!(Borrow_typescript_definition(), quote!{
        {"raw": any, "cow": any,}
    }.to_string());
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

    // TODO raw should be string(!)
    assert_eq!(Point_typescript_definition(), quote!{
        {"X": number, "Y": number,}
    }.to_string());
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
    
    assert_eq!(Enum_typescript_definition(), quote!{
        | {"tag": "Var1", "fields": boolean,}
        | {"tag": "Var2", "fields": number,}
        | {"tag": "Var3", "fields": string,}
    }.to_string());
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

    assert_eq!(Enum_typescript_definition(), quote!{
        | {"tag": "V1",}
        | {"tag": "V2",}
        | {"tag": "V3",}
    }.to_string());
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

    assert_eq!(Enum_typescript_definition(), quote!{
        | {"tag": "V1", "fields": [number, string,],}
        | {"tag": "V2", "fields": [number, boolean,],}
        | {"tag": "V3", "fields": [number, number,],}
    }.to_string());
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
    
    assert_eq!(Enum_typescript_definition(), quote!{
        | {"tag": "V1", "fields": { "Foo": boolean, }, }
        | {"tag": "V2", "fields": { "Bar": number, "Baz": number, }, }
        | {"tag": "V3", "fields": { "Quux": string, }, }
    }.to_string());
}
