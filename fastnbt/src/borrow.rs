//! This module contains types enabling 'zero-copy' capture of the array NBT
//! types. These types retain a reference to the input data when deserializing,
//! meaning the input has to live as long as the deserialized object. This can
//! be hard to manage, but offers potential performance improvements. Measure!
//! Usually the dominating factor in deserialization is decompressing the NBT
//! data.
//!
//! The [`ByteArray`], [`IntArray`], and [`LongArray`] types are the types to
//! use in your own data structures. They all implement an `iter()` method to
//! allow you to iterate over the data they contain.
//!
//! For versions that own their data, see
//! `fasnbt::{`[`ByteArray`][`crate::ByteArray`],
//! [`IntArray`][`crate::IntArray`], [`LongArray`][`crate::LongArray`]`}`.
//!
//! The `iter()` methods return an iterator to the values read on demand from an
//! internal reference to the input data.
//!
//! # Example
//!
//! ```no_run
//! use fastnbt::borrow::LongArray;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! #[serde(rename_all = "PascalCase")]
//! pub struct Section<'a> {
//!     #[serde(borrow)]
//!     pub block_states: Option<LongArray<'a>>,
//!     pub y: i8,
//! }
//!
//!# fn main(){
//!     let buf: &[u8] = unimplemented!("get a buffer from somewhere");
//!     let section: Section = fastnbt::de::from_bytes(buf).unwrap();
//!     let states = section.block_states.unwrap();
//!
//!     for long in states.iter() {
//!         // do something
//!     }
//!# }

use std::{borrow::Cow, fmt};

use byteorder::{BigEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::{CompTag, BYTE_ARRAY_TAG, INT_ARRAY_TAG, LONG_ARRAY_TAG};

/// ByteArray can be used to deserialize the NBT data of the same name. This
/// borrows from the original input data when deserializing. The carving masks
/// in a chunk use this type, for example.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ByteArray<'a> {
    tag: CompTag<BYTE_ARRAY_TAG>,
    data: &'a [u8],
}

impl<'a> ByteArray<'a> {
    /// Create an iterator over the bytes.
    pub fn iter(&self) -> ByteIter<'a> {
        ByteIter(*self)
    }
}

pub struct ByteIter<'a>(ByteArray<'a>);

impl<'a> Iterator for ByteIter<'a> {
    type Item = i8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.data.read_i8().ok()
    }
}

/// IntArray can be used to deserialize the NBT data of the same name. This
/// borrows from the original input data when deserializing. Biomes in the chunk
/// format are an example of this data type.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct IntArray<'a> {
    tag: CompTag<INT_ARRAY_TAG>,
    data: &'a [u8],
}

impl<'a> IntArray<'a> {
    /// Create an iterator over the i32s
    pub fn iter(&self) -> IntIter<'a> {
        IntIter(*self)
    }
}

pub struct IntIter<'a>(IntArray<'a>);

impl<'a> Iterator for IntIter<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.data.read_i32::<BigEndian>().ok()
    }
}

/// LongArray can be used to deserialize the NBT data of the same name. This
/// borrows from the original input data when deserializing. Block states
/// (storage of all the blocks in a chunk) are an exmple of when this is used.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct LongArray<'a> {
    tag: CompTag<LONG_ARRAY_TAG>,
    data: &'a [u8],
}

impl<'a> LongArray<'a> {
    /// Create an iterator over the i64s
    pub fn iter(&self) -> LongIter<'a> {
        LongIter(*self)
    }
}

pub struct LongIter<'a>(LongArray<'a>);

impl<'a> Iterator for LongIter<'a> {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.data.read_i64::<BigEndian>().ok()
    }
}

struct CowStr<'a>(Cow<'a, str>);

impl<'de> serde::Deserialize<'de> for CowStr<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(CowStrVisitor)
    }
}

struct CowStrVisitor;

impl<'de> serde::de::Visitor<'de> for CowStrVisitor {
    type Value = CowStr<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    // Borrowed directly from the input string, which has lifetime 'de
    // The input must outlive the resulting Cow.
    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CowStr(Cow::Borrowed(value)))
    }

    // A string that currently only lives in a temporary buffer -- we need a copy
    // (Example: serde is reading from a BufRead)
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CowStr(Cow::Owned(value.to_owned())))
    }

    // An optimisation of visit_str for situations where the deserializer has
    // already taken ownership. For example, the string contains escaped characters.
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CowStr(Cow::Owned(value)))
    }
}

pub fn deserialize_cow_str<'de, D>(deserializer: D) -> Result<Cow<'de, str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let wrapper = CowStr::deserialize(deserializer)?;
    Ok(wrapper.0)
}
