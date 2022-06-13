use std::collections::HashMap;
use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{borrow, ByteArray, IntArray, LongArray};

/// Value is a complete NBT value. It owns its data. Compounds and Lists are
/// resursively deserialized. This type takes care to preserve all the
/// information from the original NBT, with the exception of the name of the
/// root compound (which is usually the empty string).
///
/// ```no_run
/// # use fastnbt::Value;
/// # use fastnbt::error::Result;
/// # use std::collections::HashMap;
/// #
/// # fn main() -> Result<()> {
/// #   let mut buf = vec![];
///     let compound: HashMap<String, Value> = fastnbt::de::from_bytes(buf.as_slice())?;
///     match compound["DataVersion"] {
///         Value::Int(ver) => println!("Version: {}", ver),
///         _ => {},
///     }
///     println!("{:#?}", compound);
/// #   Ok(())
/// # }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Value {
    #[serde(deserialize_with = "strict_i8")]
    Byte(i8),
    #[serde(deserialize_with = "strict_i16")]
    Short(i16),
    #[serde(deserialize_with = "strict_i32")]
    Int(i32),
    Long(i64),
    Double(f64),
    Float(f32),
    String(String),
    ByteArray(ByteArray),
    IntArray(IntArray),
    LongArray(LongArray),
    List(Vec<Value>),
    Compound(HashMap<String, Value>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum BorrowedValue<'a> {
    #[serde(deserialize_with = "strict_i8")]
    Byte(i8),
    #[serde(deserialize_with = "strict_i16")]
    Short(i16),
    #[serde(deserialize_with = "strict_i32")]
    Int(i32),
    Long(i64),
    Double(f64),
    Float(f32),
    #[serde(borrow)]
    String(Cow<'a, str>),
    #[serde(borrow)]
    ByteArray(borrow::ByteArray<'a>),
    #[serde(borrow)]
    IntArray(borrow::IntArray<'a>),
    #[serde(borrow)]
    LongArray(borrow::LongArray<'a>),
    #[serde(borrow)]
    List(Vec<BorrowedValue<'a>>),
    #[serde(borrow)]
    Compound(HashMap<String, BorrowedValue<'a>>),
}

fn strict_i8<'de, D>(de: D) -> std::result::Result<i8, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct StrictI8Visitor;
    impl<'de> serde::de::Visitor<'de> for StrictI8Visitor {
        type Value = i8;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expecting exactly i8")
        }

        fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }

    de.deserialize_i8(StrictI8Visitor)
}

fn strict_i16<'de, D>(de: D) -> std::result::Result<i16, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct Stricti16Visitor;
    impl<'de> serde::de::Visitor<'de> for Stricti16Visitor {
        type Value = i16;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expecting exactly i16")
        }

        fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }

    de.deserialize_i16(Stricti16Visitor)
}

fn strict_i32<'de, D>(de: D) -> std::result::Result<i32, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct Stricti32Visitor;
    impl<'de> serde::de::Visitor<'de> for Stricti32Visitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expecting exactly i32")
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }

    de.deserialize_i32(Stricti32Visitor)
}
