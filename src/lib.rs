mod traits;
mod macros;
pub use traits::{
    BigArray,
    RequestBuilder,
    HandshakeProtocol,
    AsyncExecutor,
};
use serde::{Serialize, Deserialize};
pub use bincode::{
    config::{
        BigEndian,
        LittleEndian,
        Varint,
        Fixint,
        Limit,
        NoLimit,
    },
    Encode,
    Decode,
};

pub use macros::A64;

pub const STANDARD_CONFIG: bincode::config::Configuration<LittleEndian, Fixint> = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .with_no_limit();

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
    (@protocol $protocol_name:ident, [$($path:ident)*],
        handshake $handshake_name:ident {
            req: $req_ty:ty,
            ack: $ack_ty:ty $(,)?
        }
        $($rest:tt)*
    ) => {
        
        #[derive(Debug, Serialize, Encode, Decode, Deserialize, Clone)]
        pub struct $handshake_name {
            pub req: $req_ty,
            pub ack: $ack_ty,
        }
        
        handshake_protocol!(@protocol @protocol_name, [$($path)*], $($rest)*);
    };
    (@protocol $protocol_name:ident, [$($path:ident)*],
        protocol $nested_protocol_name:ident {
            $($nested_body:tt)*
        }
        $($rest:tt)*
    ) => {
        handshake_protocol!(@protocol $nested_protocol_name, [$($path)* $protocol_name], $($nested_body)*);
        handshake_protocol!(@protocol $protocol_name, [$($path)*], $($rest)*);
    };
        
    (@protocol $protocol_name:ident, [$($path:ident)*],)
    => {
        pub enum $protocol_name {
            $(
                $path($path),
            )*
        }
        
        impl $protocol_name {
            fn list_protocol_types() -> Vec<(String, String)> {
                vec![
                    $(
                        (stringify!($req_ty).to_string(),
                        stringify!($ack_ty).to_string())
                    ),+
                ]       
            }
            fn list_handshakes() -> Vec<String> {
                vec![
                    $(
                        stringify!($handshake_name).to_string()
                    ),+
                ]       
            }
        }
        
        impl RequestBuilder for $protocol_name {
            
            // Deserialize request message from bytes
            fn req_decode(handshake: &str, data: &[u8]) -> Self {
                match handshake {
                    $(
                        stringify!($handshake_name) => {
                            let (req, size): ($req_ty, usize) = bincode::decode_from_slice(data, STANDARD_CONFIG).unwrap();
                            Self::$handshake_name {
                                req,
                                ack: None
                            }
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
                        $protocol_name::$handshake_name { req, .. } => {
                            bincode::encode_to_vec(req, STANDARD_CONFIG).unwrap()
                        }
                    ),+
                }
            }

            // Serialize response message into bytes
            fn ack_encode(&self) -> Vec<u8> {
                match self {
                    $(
                        $protocol_name::$handshake_name { ack, .. } => {
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
                        $protocol_name::$handshake_name { ack, .. } => {
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