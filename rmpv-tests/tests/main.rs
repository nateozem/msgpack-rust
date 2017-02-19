extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate rmp_serde as rmps;
extern crate rmpv;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use rmpv::Value;

fn test_encode_ok<T>(tc: &[(T, Value)])
    where T: Debug + PartialEq + Serialize + Deserialize
{
    for &(ref var, ref val) in tc {
        // Serialize part.
        // Test that `T` -> `[u8]` equals to serialization from `Value` -> `[u8]`.
        let buf_from_var = rmps::to_vec(var).unwrap();
        let buf_from_val = rmps::to_vec(val).unwrap();
        assert_eq!(buf_from_var, buf_from_val);

        // Test that `T` -> `Value` equals with the given `Value`.
        let val_from_var = rmpv::ext::to_value(var).unwrap();
        assert_eq!(val, &val_from_var);

        // Deserialize part.
        // Test that `[u8]` -> `T` equals with the given `T`.
        let var_from_buf: T = rmps::from_slice(&buf_from_var[..]).unwrap();
        assert_eq!(var, &var_from_buf);

        // Test that `[u8]` -> `Value` equals with the given `Value`.
        let val_from_buf: Value = rmps::from_slice(&buf_from_var[..]).unwrap();
        assert_eq!(val, &val_from_buf);

        // Test that `Value` -> `T` equals with the given `T`.
        let var_from_val: T = rmpv::ext::from_value(val_from_buf).unwrap();
        assert_eq!(var, &var_from_val);
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
        (0, Value::I64(0)), // TODO: What to do if we encode signed 128 (3 bytes) as unsigned 128 (2 bytes) (for compact)?
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
