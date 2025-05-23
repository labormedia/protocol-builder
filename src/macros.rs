use serde::{ Serialize, Serializer};

// A helper to auto-derive serialization/deserialization
#[macro_export]
macro_rules! handshake_protocol {
    (
        protocol $protocol_name:ident {
            $($body:tt)*
        }
    ) => {
        use std::sync::Arc;
        use std::sync::RwLock;
        
        #[derive(Debug, Serialize, Deserialize, Decode, Encode)]
        pub enum SchemeVariants
        {
            Handshake,
            Protocol
        }
        
        handshake_protocol!(@protocol $protocol_name, [], $($body)*);
    };

    // Recursive protocol parsing
    (@protocol $protocol_name:ident, [$($path:ident)*],
        handshake $handshake_name:ident {
            req: $req_ty:ty,
            ack: $ack_ty:ty $(,)?
        }
        $($rest:tt)*
    ) => {
        // Define handshake enum
        #[derive(Debug, Serialize, Deserialize, Decode, Encode)]
        pub struct $handshake_name {
            pub req: $req_ty,
            pub ack: Option<$ack_ty>,
        }
        
        impl HandshakeGetter for $handshake_name {
            type Req = $req_ty;
            type Ack = $ack_ty;
            fn get_req_ref(&self) -> &Self::Req { &self.req }
            fn get_ack_ref(&self) -> &Option<Self::Ack> { &self.ack }
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
        #[derive(Debug, Serialize, Deserialize, Decode)]
        pub enum $protocol_name {
            $(
                $path(Arc<RwLock<$path>>)
            ),*
        }

        impl $protocol_name {
            /*fn list_protocol_types() -> Vec<(String, String)> {
                vec![
                    $(
                        (stringify!($req_ty).to_string(),
                        stringify!($ack_ty).to_string())
                    ),+
                ]       
            }*/
            fn list_handshakes() -> Vec<String> {
                vec![
                    $(
                        stringify!($path).to_string()
                    ),*
                ]       
            }
        }

        impl RequestBuilder for $protocol_name {
            
            // Deserialize request message from bytes
            fn req_decode(handshake: &str, data: &[u8]) -> Self {
                match handshake {
                    $(
                        $path => {
                            let (req, size): (_, usize) = bincode::decode_from_slice(data, STANDARD_CONFIG).unwrap();
                            $protocol_name::$path( Arc::new(
                                (RwLock::new($path {
                                    req,
                                    ack: None
                                }))
                            ))

                        }
                    ),*
                    _ => panic!("Unknown handshake request"),
                }
            }
        }
        impl HandshakeProtocol for $protocol_name {
            // Serialize request message into bytes
            fn req_encode(&self) -> Vec<u8> {
                match self {
                    $(
                        $protocol_name::$path( lock ) => {
                            let lock_clone = lock.clone();
                            let (path) = lock_clone.read().unwrap();
                            bincode::encode_to_vec(path.get_req_ref(), STANDARD_CONFIG).unwrap()
                        },
                    )*
                }
            }

            // Serialize response message into bytes
            fn ack_encode(&self) -> Vec<u8> {
                match self {
                    $(
                        $protocol_name::$path( lock ) => {
                            let lock_clone = lock.clone();
                            let (path) = lock_clone.read().unwrap();
                            match path.get_ack_ref() {
                                Some(ack) => bincode::encode_to_vec(ack, STANDARD_CONFIG).unwrap(),
                                None => Vec::new()
                            }
                        }
                    ),*
                }
            }

            // Deserialize response message from bytes
            fn ack_decode(&mut self, data: &[u8]) {
                match self {
                    $(
                        $protocol_name::$path(lock) => {
                            let lock_clone = lock.clone();
                            let mut path_option = lock_clone.write().unwrap();
                            
                            let (decoded, _) = bincode::decode_from_slice(data, STANDARD_CONFIG).unwrap();
                            
                            path_option.ack = Some(decoded);
                        }
                    ),*
                    _ => {}
                }
            }
        }
    };
}

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