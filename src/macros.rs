use serde::{ Serialize, Serializer};

pub fn serialize_array<S, T>(array: &[T], serializer: S) -> Result<S::Ok, S::Error>
where S: Serializer, T: Serialize {
	array.serialize(serializer)
}

#[macro_export]
macro_rules! serde_array { ($m:ident, $n:expr) => {
	pub mod $m {
		use std::{ptr, mem};
		use serde::{Deserialize, Deserializer, de};
		pub use $crate::macros::serialize_array as serialize;
		use super::*;

		pub fn deserialize<'de, D, T>(deserializer: D) -> Result<[T; $n], D::Error>
		where D: Deserializer<'de>, T: Deserialize<'de> + 'de {
			let slice: Vec<T> = Deserialize::deserialize(deserializer)?;
			if slice.len() != $n {
				return Err(de::Error::custom("input slice has wrong length"));
			}
			unsafe {
				let mut result: [T; $n] = mem::uninitialized();
				for (src, dst) in slice.into_iter().zip(&mut result[..]) {
					ptr::write(dst, src);
				}
				Ok(result)
			}
		}
	}
}}


serde_array!(A64, 64);