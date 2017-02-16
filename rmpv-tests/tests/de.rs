extern crate serde;
extern crate rmp_serde;
extern crate rmpv;

use std::fmt::Debug;
use std::io::Cursor;

use serde::Deserialize;

use rmp_serde::Deserializer;
use rmpv::Value;
use rmpv::decode::read_value;
use rmpv::ext::from_value;

#[test]
fn pass_option_some_value() {
    let buf = [0x1f];
    let cur = Cursor::new(&buf[..]);

    let mut de = Deserializer::new(cur);
    let actual = Deserialize::deserialize(&mut de).unwrap();
    assert_eq!(Value::from(31), actual);
}

#[test]
fn pass_option_none_value() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut de = Deserializer::new(cur);
    let actual = Deserialize::deserialize(&mut de).unwrap();
    assert_eq!(Value::Nil, actual);
}

#[test]
fn pass_map_value() {
    let buf = [0x82 /* 2 (size) */, 0xa3, 0x69, 0x6e, 0x74 /* 'int' */, 0xcc,
               0x80 /* 128 */, 0xa3, 0x6b, 0x65, 0x79 /* 'key' */, 0x2a /* 42 */];
    let cur = Cursor::new(&buf[..]);

    let mut de = Deserializer::new(cur);
    let actual = Deserialize::deserialize(&mut de).unwrap();
    let expected = Value::Map(vec![
        (Value::String("int".into()), Value::from(128)),
        (Value::String("key".into()), Value::from(42)),
    ]);

    assert_eq!(expected, actual);
}

// TODO: Merge three of them.
#[test]
fn pass_bin8_into_bytebuf_value() {
    let buf = [0xc4, 0x02, 0xcc, 0x80];
    let cur = Cursor::new(&buf[..]);

    let mut de = Deserializer::new(cur);
    let actual = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Value::Binary(vec![0xcc, 0x80]), actual)
}

#[test]
fn pass_bin16_into_bytebuf_value() {
    let buf = [0xc5, 0x00, 0x02, 0xcc, 0x80];
    let cur = Cursor::new(&buf[..]);

    let mut de = Deserializer::new(cur);
    let actual = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Value::Binary(vec![0xcc, 0x80]), actual);
}

#[test]
fn pass_bin32_into_bytebuf_value() {
    let buf = [0xc6, 0x00, 0x00, 0x00, 0x02, 0xcc, 0x80];
    let cur = Cursor::new(&buf[..]);

    let mut de = Deserializer::new(cur);
    let actual = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Value::Binary(vec![0xcc, 0x80]), actual);
}

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
