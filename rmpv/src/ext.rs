use std::error;
use std::fmt::{self, Display, Formatter};
use std::vec::IntoIter;

use serde::{self, Serialize, Serializer, Deserialize};
use serde::bytes::Bytes;
use serde::de::{self, DeserializeSeed, SeqVisitor, Unexpected, Visitor};
use serde::de::value::ValueDeserializer;
use serde::ser::{SerializeSeq, SerializeMap};

use Value;

impl Serialize for Value {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            Value::Nil => s.serialize_unit(),
            Value::Boolean(v) => s.serialize_bool(v),
            Value::I64(v) => s.serialize_i64(v),
            Value::U64(v) => s.serialize_u64(v),
            Value::F32(v) => s.serialize_f32(v),
            Value::F64(v) => s.serialize_f64(v),
            Value::String(ref v) => s.serialize_str(v),
            Value::Binary(ref v) => Bytes::from(v).serialize(s),
            Value::Array(ref array) => {
                let mut state = s.serialize_seq(Some(array.len()))?;
                for item in array {
                    state.serialize_element(item)?;
                }
                state.end()
            }
            Value::Map(ref map) => {
                let mut state = s.serialize_map(Some(map.len()))?;
                for &(ref key, ref val) in map {
                    state.serialize_entry(key, val)?;
                }
                state.end()
            }
            Value::Ext(ty, ref buf) => {
                let mut state = s.serialize_seq(Some(2))?;
                state.serialize_element(&ty)?;
                state.serialize_element(buf)?;
                state.end()
            }
        }
    }
}

impl Deserialize for Value {
    #[inline]
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
        where D: de::Deserializer
    {
        struct ValueVisitor;

        impl serde::de::Visitor for ValueVisitor {
            type Value = Value;

            fn expecting(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
                fmt.write_str("any valid MessagePack value")
            }

            #[inline]
            fn visit_some<D>(self, de: D) -> Result<Value, D::Error>
                where D: de::Deserializer
            {
                Deserialize::deserialize(de)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Boolean(value))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::U64(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                if value < 0 {
                    Ok(Value::I64(value))
                } else {
                    Ok(Value::U64(value as u64))
                }
            }

            #[inline]
            fn visit_f32<E>(self, value: f32) -> Result<Value, E> {
                Ok(Value::F32(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::F64(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
                where E: serde::de::Error
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_seq<V>(self, visitor: V) -> Result<Value, V::Error>
                where V: serde::de::SeqVisitor
            {
                let values: Vec<Value> = try!(serde::de::impls::VecVisitor::new()
                    .visit_seq(visitor));
                let values = values.into_iter().collect();

                Ok(Value::Array(values))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(Value::Binary(v.to_owned()))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut pairs = vec![];

                loop {
                    let key: Option<Value> = try!(visitor.visit_key());
                    if let Some(key) = key {
                        let value: Value = try!(visitor.visit_value());

                        pairs.push((key, value));
                    } else {
                        break;
                    }
                }

                Ok(Value::Map(pairs))
            }
        }

        de.deserialize(ValueVisitor)
    }
}

#[derive(Debug)]
pub enum Error {
    Syntax(String),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Syntax(ref err) => write!(fmt, "{}: {}", error::Error::description(self), err)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "error while decoding value"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Syntax(..) => None,
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Syntax(format!("{}", msg))
    }
}

pub struct Deserializer {}

impl de::Deserializer for Value {
    type Error = Error;

    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor
    {
        match self {
            Value::Nil => visitor.visit_unit(),
            Value::Boolean(v) => visitor.visit_bool(v),
            Value::U64(v) => visitor.visit_u64(v),
            Value::I64(v) => visitor.visit_i64(v),
            Value::F32(v) => visitor.visit_f32(v),
            Value::F64(v) => visitor.visit_f64(v),
            Value::String(v) => visitor.visit_string(v),
            Value::Binary(v) => visitor.visit_byte_buf(v),
            Value::Array(v) => {
                let len = v.len();
                let mut de = SeqDeserializer::new(v);
                let seq = visitor.visit_seq(&mut de)?;
                if de.iter.len() == 0 {
                    Ok(seq)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in array"))
                }
            }
            Value::Map(v) => {
                let len = v.len();
                let mut de = MapDeserializer::new(v);
                let map = visitor.visit_map(&mut de)?;
                if de.iter.len() == 0 {
                    Ok(map)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in map"))
                }
            }
            Value::Ext(..) => {
                // TODO: [i8, [u8]] can be represented as:
                //      - (0i8, Vec<u8>),
                //      - struct F(i8, Vec<u8>),
                //      - struct F {ty: i8, val: Vec<u8>}
                //      - enum F{ A(Vec<u8>), B { name: Vec<u8> } }
                unimplemented!();
            }
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor
    {
        if let Value::Nil = self {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn deserialize_enum<V>(self, _name: &str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        match self {
            Value::Array(v) => {
                let mut iter = v.into_iter();

                if !(iter.len() == 1 || iter.len() == 2) {
                    return Err(de::Error::invalid_value(Unexpected::Seq, &"array with one or two elements"));
                }

                let id = match iter.next() {
                    Some(id) => from_value(id)?,
                    None => {
                        return Err(de::Error::invalid_value(Unexpected::Seq, &"array with one or two elements"));
                    }
                };

                visitor.visit_enum(EnumDeserializer {
                    id: id,
                    value: iter.next(),
                })
            }
            other => {
                Err(de::Error::invalid_type(other.unexpected(), &"array, map or int"))
            }
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        seq_fixed_size bytes byte_buf map unit_struct tuple_struct struct
        struct_field tuple ignored_any
    }
}

struct SeqDeserializer {
    iter: IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl SeqVisitor for SeqDeserializer {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: de::DeserializeSeed
    {
        match self.iter.next() {
            Some(val) => seed.deserialize(val).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl de::Deserializer for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = visitor.visit_seq(&mut self)?;
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(de::Error::invalid_length(len, &"fewer elements in array"))
            }
        }
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }
}

struct MapDeserializer {
    val: Option<Value>,
    iter: IntoIter<(Value, Value)>,
}

impl MapDeserializer {
    fn new(map: Vec<(Value, Value)>) -> Self {
        MapDeserializer {
            val: None,
            iter: map.into_iter(),
        }
    }
}

impl de::MapVisitor for MapDeserializer {
    type Error = Error;

    fn visit_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: DeserializeSeed
    {
        match self.iter.next() {
            Some((key, val)) => {
                self.val = Some(val);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn visit_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeSeed
    {
        match self.val.take() {
            Some(val) => seed.deserialize(val),
            None => Err(de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl de::Deserializer for MapDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }
}

struct EnumDeserializer {
    id: u32,
    value: Option<Value>,
}

impl de::EnumVisitor for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where V: de::DeserializeSeed
    {
        let variant = self.id.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl de::VariantVisitor for VariantDeserializer {
    type Error = Error;

    fn visit_unit(self) -> Result<(), Error> {
        // Can accept only [u32].
        match self.value {
            Some(value) => de::Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value, Error>
        where T: de::DeserializeSeed,
    {
        // Can accept both [u32, T] and [u32, [T]] cases.
        match self.value {
            Some(Value::Array(v)) => {
                let mut iter = v.into_iter();
                let val = match iter.next() {
                    Some(val) => seed.deserialize(val),
                    None => return Err(de::Error::invalid_value(Unexpected::Seq, &"array with one element")),
                };

                if iter.next().is_some() {
                    Err(de::Error::invalid_value(Unexpected::Seq, &"array with one element"))
                } else {
                    val
                }
            }
            Some(value) => seed.deserialize(value),
            None => Err(de::Error::invalid_type(Unexpected::UnitVariant, &"newtype variant")),
        }
    }

    fn visit_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        // Can accept [u32, [T...]].
        match self.value {
            Some(Value::Array(v)) => {
                de::Deserializer::deserialize(SeqDeserializer::new(v), visitor)
            }
            Some(other) => Err(de::Error::invalid_type(other.unexpected(), &"tuple variant")),
            None => Err(de::Error::invalid_type(Unexpected::UnitVariant, &"tuple variant"))
        }
    }

    fn visit_struct<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.value {
            Some(Value::Array(v)) => {
                de::Deserializer::deserialize(SeqDeserializer::new(v), visitor)
            }
            Some(Value::Map(v)) => {
                de::Deserializer::deserialize(MapDeserializer::new(v), visitor)
            }
            Some(other) => Err(de::Error::invalid_type(other.unexpected(), &"struct variant")),
            _ => Err(de::Error::invalid_type(Unexpected::UnitVariant, &"struct variant"))
        }
    }
}

trait ValueExt {
    fn unexpected(&self) -> Unexpected;
}

impl ValueExt for Value {
    fn unexpected(&self) -> Unexpected {
        match *self {
            Value::Nil => Unexpected::Unit,
            Value::Boolean(v) => Unexpected::Bool(v),
            Value::U64(v) => Unexpected::Unsigned(v),
            Value::I64(v) => Unexpected::Signed(v),
            Value::F32(v) => Unexpected::Float(v as f64),
            Value::F64(v) => Unexpected::Float(v),
            Value::String(ref v) => Unexpected::Str(v),
            Value::Binary(ref v) => Unexpected::Bytes(v),
            Value::Array(..) => Unexpected::Seq,
            Value::Map(..) => Unexpected::Map,
            Value::Ext(..) => Unexpected::Seq,
        }
    }
}

pub fn from_value<T>(val: Value) -> Result<T, Error>
    where T: de::Deserialize
{
    de::Deserialize::deserialize(val)
}

// pub fn to_value<T: Serialize>(value: T) -> Result<Value, Error> {
//     value.serialize(Serializer)
// }
