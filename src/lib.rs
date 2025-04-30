mod traits;
mod macros;
use traits::BigArray;

use serde::{Serialize, Deserialize};
use bincode;

pub use macros::A64;

pub trait HandshakeProtocol {
    fn serialize_request(&self) -> Vec<u8>;
    fn deserialize_request(handshake: &str, data: &[u8]) -> Self;
    fn serialize_response(&self) -> Vec<u8>;
    fn deserialize_response(&mut self, data: &[u8]);
}

// A helper to auto-derive serialization/deserialization
#[macro_export]
macro_rules! handshake_protocol {
    (
        protocol $protocol_name:ident {
            $(
                handshake $handshake_name:ident {
                    request: $req_ty:ty,
                    response: $res_ty:ty
                }
            ),+ $(,)?
        }
    ) => {
        use protocol_builder::HandshakeProtocol;
        
        #[derive(Debug)]
        pub enum $protocol_name {
            $(
                $handshake_name {
                    request: $req_ty,
                    response: $res_ty,
                }
            ),+
        }

        impl HandshakeProtocol for $protocol_name {
            // Serialize request message into bytes
            fn serialize_request(&self) -> Vec<u8> {
                match self {
                    $(
                        $protocol_name::$handshake_name { request, .. } => {
                            bincode::serialize(request).unwrap()
                        }
                    ),+
                }
            }

            // Deserialize request message from bytes
            fn deserialize_request(handshake: &str, data: &[u8]) -> Self {
                match handshake {
                    $(
                        stringify!($handshake_name) => {
                            let request: $req_ty = bincode::deserialize(data).unwrap();
                            Self::$handshake_name {
                                request,
                                response: Default::default()
                            }
                        }
                    ),+
                    _ => panic!("Unknown handshake request"),
                }
            }

            // Serialize response message into bytes
            fn serialize_response(&self) -> Vec<u8> {
                match self {
                    $(
                        $protocol_name::$handshake_name { response, .. } => {
                            bincode::serialize(response).unwrap()
                        }
                    ),+
                }
            }

            // Deserialize response message from bytes
            fn deserialize_response(&mut self, data: &[u8]) {
                match self {
                    $(
                        $protocol_name::$handshake_name { response, .. } => {
                            *response = bincode::deserialize(data).unwrap();
                        }
                    ),+
                }
            }
        }
    };
}