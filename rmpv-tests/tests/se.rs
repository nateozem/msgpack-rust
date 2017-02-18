extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate rmp_serde;
extern crate rmpv;

use std::fmt::Debug;

use serde::Serialize;
use serde::bytes::{Bytes, ByteBuf};

use rmp_serde::Serializer;
use rmpv::Value;
use rmpv::ext::to_value;

fn test_encode_ok<T>(tc: &[(T, Value)])
    where T: Debug + PartialEq + Serialize
{
    for &(ref c, ref val) in tc {
        assert_eq!(val, &to_value(&c).unwrap());
    }
}

#[test]
fn pass_write_nil() {
    test_encode_ok(&[
        ((), Value::Nil),
    ]);
}

#[test]
fn pass_write_bool() {
    test_encode_ok(&[
        (true, Value::Boolean(true)),
        (false, Value::Boolean(false)),
    ]);
}

#[test]
fn pass_write_sint() {
    test_encode_ok(&[
        (0, Value::I64(0)),
        (127i8, Value::I64(127)),
        (-128i8, Value::I64(-128)),
    ]);

    test_encode_ok(&[
        (0, Value::I64(0)),
        (32767i16, Value::I64(32767)),
        (-32768i16, Value::I64(-32768)),
    ]);

    test_encode_ok(&[
        (0, Value::I64(0)),
        (9223372036854775807i64, Value::I64(9223372036854775807)),
        (-9223372036854775808i64, Value::I64(-9223372036854775808)),
    ]);
}

#[test]
fn pass_write_uint() {
    test_encode_ok(&[
        (0, Value::U64(0)),
        (255u8, Value::U64(255)),
    ]);

    test_encode_ok(&[
        (0, Value::U64(0)),
        (65535u16, Value::U64(65535)),
    ]);

    test_encode_ok(&[
        (0, Value::U64(0)),
        (18446744073709551615u64, Value::U64(18446744073709551615u64)),
    ]);
}

#[test]
fn pass_write_float() {
    test_encode_ok(&[
        (3.1415f32, Value::F32(3.1415)),
    ]);

    test_encode_ok(&[
        (3.1415f64, Value::F64(3.1415)),
    ]);
}

#[test]
fn pass_write_char() {
    test_encode_ok(&[
        ('c', Value::String("c".into())),
    ]);
}

#[test]
fn pass_write_string() {
    test_encode_ok(&[
        ("le message", Value::String("le message".into())),
    ]);

    test_encode_ok(&[
        ("le message".to_string(), Value::String("le message".into())),
    ]);
}

#[test]
fn pass_write_bytes() {
    test_encode_ok(&[
        (Bytes::new(&[0, 1, 2]), Value::Binary(vec![0, 1, 2])),
    ]);

    test_encode_ok(&[
        (ByteBuf::from(&[0, 1, 2][..]), Value::Binary(vec![0, 1, 2])),
    ]);
}

#[test]
fn pass_write_unit_struct() {
    #[derive(Debug, PartialEq, Serialize)]
    struct Unit;

    test_encode_ok(&[
        (Unit, Value::Nil),
    ]);
}

#[test]
fn pass_write_enum() {
    #[derive(Debug, PartialEq, Serialize)]
    enum Enum {
        Unit,
        Newtype(String),
        Tuple(String, u32),
        Struct { name: String, age: u32 },
    }

    test_encode_ok(&[
        (Enum::Unit, Value::Nil), // TODO: Need round-trip cases.
    ]);
}

#[test]
fn pass_value_nil() {
    let mut buf = Vec::new();

    Value::Nil.serialize(&mut Serializer::new(&mut buf)).unwrap();

    assert_eq!(vec![0xc0], buf);
}

#[test]
fn pass_value_bool() {
    let mut buf = Vec::new();
    {
        let mut encoder = Serializer::new(&mut buf);

        let val = Value::from(true);
        val.serialize(&mut encoder).unwrap();

        let val = Value::from(false);
        val.serialize(&mut encoder).unwrap();
    }

    assert_eq!(vec![0xc3, 0xc2], buf);
}

#[test]
fn pass_value_usize() {
    check_ser(Value::from(255usize), &[0xcc, 0xff]);
}

#[test]
fn pass_value_isize() {
    check_ser(Value::from(-128isize), &[0xd0, 0x80]);
}

#[test]
fn pass_value_f32() {
    check_ser(Value::from(3.4028234e38_f32), &[0xca, 0x7f, 0x7f, 0xff, 0xff]);
}

#[test]
fn pass_value_f64() {
    check_ser(Value::from(42.0), &[0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn pass_value_string() {
    check_ser(Value::String("le message".into()),
        &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65]);
}

#[test]
fn pass_value_bin() {
    check_ser(Value::Binary(vec![0xcc, 0x80]), &[0xc4, 0x02, 0xcc, 0x80]);
}

#[test]
fn pass_value_array() {
    check_ser(Value::Array(vec![Value::String("le".into()), Value::String("shit".into())]),
        &[0x92, 0xa2, 0x6c, 0x65, 0xa4, 0x73, 0x68, 0x69, 0x74]);
}

#[test]
fn pass_value_map() {
    let val = Value::Map(vec![
        (Value::from(0), Value::String("le".into())),
        (Value::from(1), Value::String("shit".into())),
    ]);

    let out = [
        0x82, // 2 (size)
        0x00, // 0
        0xa2, 0x6c, 0x65, // "le"
        0x01, // 1
        0xa4, 0x73, 0x68, 0x69, 0x74, // "shit"
    ];

    check_ser(val, &out);
}

fn check_ser<T>(val: T, expected: &[u8])
    where T: Serialize
{
    let mut buf = Vec::new();
    val.serialize(&mut Serializer::new(&mut buf)).unwrap();
    assert_eq!(expected, &buf[..]);
}
