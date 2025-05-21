#![feature(log_syntax)]
use serde::{ Serialize, Serializer};
trace_macros!(true);
// A helper to auto-derive serialization/deserialization
#[macro_export]
macro_rules! handshake_protocol {
    (
        protocol $protocol_name:ident {
            $($body:tt)*
        }
    ) => {
        handshake_protocol!(@protocol $protocol_name, [], $($body)*);
    };

    // Recursive protocol parsing
    (@protocol $protocol_name:ident, [$($path:ident)*],
        handshake $handshake_name:ident {
            req: $req_ty:ty,
            ack: $ack_ty:ty
        }
        $($rest:tt)*
    ) => {
        // Define handshake enum
        #[derive(Debug, Serialize, Deserialize)]
        pub struct $handshake_name {
            pub req: $req_ty,
            pub ack: Option<$ack_ty>,
        }
        handshake_protocol!(@protocol $protocol_name, [$($path)* $handshake_name], $($rest)*);
        
    };

    // Handle nested protocol
    (@protocol $protocol_name:ident, [$($path:ident)*],
        protocol $nested_protocol_name:ident {
            $($nested_body:tt)*
        }
        $($rest:tt)*
    ) => {
        // Recurse into nested protocol
        handshake_protocol!(@protocol $nested_protocol_name, [$($path)*], $($nested_body)*);
        handshake_protocol!(@protocol $protocol_name, [$($path)*], $($rest)*);
    };

    // When body is empty, define protocol enum
    (@protocol $protocol_name:ident, [$($path:ident)*],) => {
        #[derive(Debug, Serialize, Deserialize)]
        pub enum $protocol_name {
            $(
                $path($path),
            )*
        }

        impl RequestBuilder for $protocol_name {
            
            // Deserialize request message from bytes
            fn req_decode(handshake: &str, data: &[u8]) -> Self {
                match handshake {
                    $(
                        $path => {
                            let (req, size): (_, usize) = bincode::decode_from_slice(data, STANDARD_CONFIG).unwrap();
                            $protocol_name::$path( $path {
                                req,
                                ack: None
                            })
                        }
                    ),+
                    _ => panic!("Unknown handshake request"),
                }
            }
        }
        impl HandshakeProtocol for $protocol_name {
            // Serialize request message into bytes
            fn req_encode(&self) -> Vec<u8> {
                match self {
                    $(
                        $protocol_name::$path( $path { req, .. }) => {
                            bincode::encode_to_vec(req, STANDARD_CONFIG).unwrap()
                        }
                    ),+
                }
            }

            // Serialize response message into bytes
            fn ack_encode(&self) -> Vec<u8> {
                match self {
                    $(
                        $protocol_name::$path($path { ack, .. }) => {
                            match ack {
                                Some(acknowledge) => bincode::encode_to_vec(acknowledge, STANDARD_CONFIG).unwrap(),
                                None => Vec::new(),
                            }
                        }
                    ),+
                }
            }

            // Deserialize response message from bytes
            fn ack_decode(&mut self, data: &[u8]) {
                match self {
                    $(
                        $protocol_name::$path($path { ack, .. }) => {
                            let acknowledge = match bincode::decode_from_slice(data, STANDARD_CONFIG).ok() {
                                Some((encoded, _)) => Some(encoded),
                                None => None,
                            };
                            *ack = acknowledge;
                        }
                    ),+
                }
            }
        }
    };
}

pub fn serialize_array<S, T>(array: &[T], serializer: S) -> Result<S::Ok, S::Error>
where S: Serializer, T: Serialize {
	array.serialize(serializer)
}

trace_macros!(false);

#[macro_export]
macro_rules! serde_array { ($m:ident, $n:expr) => {
	pub mod $m {
		use std::{ptr, mem};
		use serde::{Deserialize, Deserializer, de};
		pub use $crate::macros::serialize_array as serialize;

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