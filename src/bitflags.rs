use bitflags::BitFlags;
use serde::de::{Error, Visitor};
use std::fmt;
use std::marker::PhantomData;

pub struct BitFlagsVisitor<T>(PhantomData<T>);

impl<T> BitFlagsVisitor<T> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<'de, T: BitFlags<Bits = u32>> Visitor<'de> for BitFlagsVisitor<T> {
	type Value = T;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("bitflags")
	}

	fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
	where
		E: Error,
	{
		T::from_bits(v).ok_or_else(|| E::custom("invalid value"))
	}
}
