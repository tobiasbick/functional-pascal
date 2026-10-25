//! Check nested SDK numbers before serde_json can replace a nonfinite float with null.

use serde::Serialize;
use serde::ser::{self, SerializeMap, SerializeSeq, Serializer};

use crate::{CodecError, ErrorKind};

#[derive(Clone, Copy)]
pub(super) struct Check(usize);

impl Check {
    pub(super) const fn new() -> Self {
        Self(0)
    }

    fn nested(self) -> Result<Self, CodecError> {
        if self.0 >= crate::raw::MAX_DEPTH {
            return Err(CodecError::new(ErrorKind::DepthExceeded));
        }
        Ok(Self(self.0 + 1))
    }
}

macro_rules! scalars {
    ($($method:ident($ty:ty)),* $(,)?) => {$(
        fn $method(self, _: $ty) -> Result<(), CodecError> { Ok(()) }
    )*};
}

impl Serializer for Check {
    type Ok = ();
    type Error = CodecError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    scalars! {
        serialize_bool(bool), serialize_i8(i8), serialize_i16(i16), serialize_i32(i32),
        serialize_i64(i64), serialize_i128(i128), serialize_u8(u8), serialize_u16(u16),
        serialize_u32(u32), serialize_u64(u64), serialize_u128(u128), serialize_char(char),
        serialize_str(&str), serialize_bytes(&[u8])
    }

    fn serialize_f32(self, value: f32) -> Result<(), CodecError> {
        self.serialize_f64(f64::from(value))
    }
    fn serialize_f64(self, value: f64) -> Result<(), CodecError> {
        if value.is_finite() {
            Ok(())
        } else {
            Err(CodecError::new(ErrorKind::NonFinite))
        }
    }
    fn serialize_none(self) -> Result<(), CodecError> {
        Ok(())
    }
    fn serialize_unit(self) -> Result<(), CodecError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), CodecError> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), CodecError> {
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), CodecError> {
        value.serialize(self)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), CodecError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        value: &T,
    ) -> Result<(), CodecError> {
        value.serialize(self)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self, CodecError> {
        self.nested()
    }
    fn serialize_tuple(self, _: usize) -> Result<Self, CodecError> {
        self.nested()
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self, CodecError> {
        self.nested()
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self, CodecError> {
        self.nested()
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self, CodecError> {
        self.nested()
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self, CodecError> {
        self.nested()
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self, CodecError> {
        self.nested()
    }
}

impl SerializeSeq for Check {
    type Ok = ();
    type Error = CodecError;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), CodecError> {
        value.serialize(*self)
    }
    fn end(self) -> Result<(), CodecError> {
        Ok(())
    }
}

macro_rules! tuples {
    ($($trait:ident($method:ident)),* $(,)?) => {$(
        impl ser::$trait for Check {
            type Ok = ();
            type Error = CodecError;
            fn $method<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), CodecError> { value.serialize(*self) }
            fn end(self) -> Result<(), CodecError> { Ok(()) }
        }
    )*};
}
tuples! { SerializeTuple(serialize_element), SerializeTupleStruct(serialize_field), SerializeTupleVariant(serialize_field) }

impl SerializeMap for Check {
    type Ok = ();
    type Error = CodecError;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), CodecError> {
        key.serialize(*self)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), CodecError> {
        value.serialize(*self)
    }
    fn end(self) -> Result<(), CodecError> {
        Ok(())
    }
}

macro_rules! structures {
    ($($trait:ident),* $(,)?) => {$(
        impl ser::$trait for Check {
            type Ok = ();
            type Error = CodecError;
            fn serialize_field<T: Serialize + ?Sized>(&mut self, _: &'static str, value: &T) -> Result<(), CodecError> { value.serialize(*self) }
            fn end(self) -> Result<(), CodecError> { Ok(()) }
        }
    )*};
}
structures! { SerializeStruct, SerializeStructVariant }
