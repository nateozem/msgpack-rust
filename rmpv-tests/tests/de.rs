extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate rmpv;

use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::Deserialize;
use serde::bytes::ByteBuf;

use rmpv::Value;
use rmpv::decode::read_value;
use rmpv::ext::from_value;

fn test_parse_ok<T>(tc: Vec<(&[u8], T)>)
    where T: Debug + PartialEq + Deserialize
{
    for (buf, val) in tc {
        let newval: Value = read_value(&mut &buf[..]).unwrap();
        let v: T = from_value(newval).unwrap();
        assert_eq!(v, val);
    }
}

#[test]
fn pass_null() {
    test_parse_ok::<()>(vec![
        (&[0xc0], ())
    ]);
}

#[test]
fn pass_bool() {
    test_parse_ok::<bool>(vec![
        (&[0xc2], false),
        (&[0xc3], true),
    ]);
}

#[test]
fn pass_u64() {
    test_parse_ok::<u64>(vec![
        (&[0x7f], 127),
        (&[0xcc, 0xff], 255),
        (&[0xcd, 0xff, 0xff], 65535),
        (&[0xce, 0xff, 0xff, 0xff, 0xff], 4294967295),
        (&[0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], 18446744073709551615u64),
    ]);
}

#[test]
fn pass_i64() {
    test_parse_ok::<i64>(vec![
        (&[0x00], 0),
        (&[0xd0, 0x7f], 127),
        (&[0xd1, 0x7f, 0xff], 32767),
        (&[0xd2, 0x7f, 0xff, 0xff, 0xff], 2147483647),
        (&[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], 9223372036854775807),
    ]);
}

#[test]
fn pass_f32() {
    test_parse_ok::<f32>(vec![
        (&[0xca, 0x7f, 0x7f, 0xff, 0xff], 3.4028234e38),
    ]);
}

#[test]
fn pass_f64() {
    test_parse_ok::<f64>(vec![
        (&[0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 42.),
    ]);
}

#[test]
fn pass_string() {
    test_parse_ok::<String>(vec![
        (&[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65], "le message".into()),
    ]);
}

#[test]
fn pass_tuple() {
    test_parse_ok::<(u8, u32)>(vec![
        (&[0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94], (42, 100500)),
    ]);
}

#[test]
fn pass_array() {
    test_parse_ok::<[u32; 2]>(vec![
        (&[0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94], [42, 100500]),
    ]);
}

#[test]
fn pass_vec() {
    test_parse_ok::<Vec<u32>>(vec![
        (&[0x92, 0x00, 0xcc, 0x80], vec![0, 128]),
        (&[0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94], vec![42, 100500]),
    ]);
}

macro_rules! treemap {
    () => {
        BTreeMap::new()
    };
    ($($k:expr => $v:expr),+) => {
        {
            let mut m = BTreeMap::new();
            $(
                m.insert($k, $v);
            )+
            m
        }
    };
}

#[test]
fn pass_map() {
    test_parse_ok::<BTreeMap<String, u32>>(
        vec![
            (
                &[0x80],
                treemap!()
            ),
            (
                &[0x82, 0xa3, 0x69, 0x6e, 0x74, 0xcc, 0x80, 0xa3, 0x6b, 0x65, 0x79, 0x2a],
                treemap!("int".into() => 128, "key".into() => 42)
            ),
        ]
    );
}

#[test]
fn pass_option() {
    test_parse_ok(vec![
        (&[0xc0], None::<String>),
        (&[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65], Some("le message".into())),
    ]);
}

#[test]
fn pass_newtype_struct() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo(Option<i32>);

    test_parse_ok(vec![
        (&[0xc0], Foo(None)),
        (&[0x04], Foo(Some(4))),
    ]);

    #[derive(Debug, PartialEq, Deserialize)]
    struct Bar(Vec<i32>);

    test_parse_ok(vec![
        (&[0x90], Bar(vec![])),
        (&[0x91, 0x04], Bar(vec![4])),
    ]);
}

#[test]
fn pass_struct() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        x: Option<i32>,
    }

    test_parse_ok(vec![
        (&[0x91, 0xc0], Foo { x: None }),
        (&[0x91, 0x04], Foo { x: Some(4) }),
        (&[0x81, 0xa1, 0x78, 0xc0], Foo { x: None }),
        (&[0x81, 0xa1, 0x78, 0x2a], Foo { x: Some(42) }),
    ]);
}

#[test]
fn pass_enum() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum Variant {
        Empty,
        New(u32),
        Tuple(u32, String),
        Struct{ age: u32, name: String },
    }

    test_parse_ok(vec![
        // [0].
        (&[0x91, 0x00], Variant::Empty),
        // [0, []].
        (&[0x92, 0x00, 0x90], Variant::Empty),
        // [1, 42].
        (&[0x92, 0x01, 0x2a], Variant::New(42)),
        // [1, [42]].
        (&[0x92, 0x01, 0x91, 0x2a], Variant::New(42)),
        // [2, [42, "name"]].
        (&[0x92, 0x02, 0x92, 0x2a, 0xa4, 0x6e, 0x61, 0x6d, 0x65], Variant::Tuple(42, "name".into())),
        // [3, [42, "name"]].
        (&[0x92, 0x03, 0x92, 0x2a, 0xa4, 0x6e, 0x61, 0x6d, 0x65], Variant::Struct { age: 42, name: "name".into() }),
        // [3, {"age": 42, "name": "name"}]
        (&[0x92, 0x03, 0x82, 0xa3, 0x61, 0x67, 0x65, 0x2a, 0xa4, 0x6e, 0x61, 0x6d, 0x65, 0xa4, 0x6e, 0x61, 0x6d, 0x65], Variant::Struct { age: 42, name: "name".into() }),
    ]);
}

#[test]
fn pass_bytebuf() {
    test_parse_ok(vec![
        (&[0xc4, 0x02, 0xcc, 0x80], ByteBuf::from(vec![0xcc, 0x80])),
        (&[0xc5, 0x00, 0x02, 0xcc, 0x80], ByteBuf::from(vec![0xcc, 0x80])),
        (&[0xc6, 0x00, 0x00, 0x00, 0x02, 0xcc, 0x80], ByteBuf::from(vec![0xcc, 0x80])),
    ]);
}
