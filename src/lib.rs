mod traits;
mod macros;
use traits::BigArray;
use serde::{Serialize, Deserialize};
pub use bincode::{
    config::{
        BigEndian,
        LittleEndian,
        Varint,
        Fixint,
        Limit,
    },
    Encode,
    Decode,
};

pub use macros::A64;

pub trait HandshakeProtocol {
    fn serialize_req(&self) -> Vec<u8>;
    fn deserialize_req(handshake: &str, data: &[u8]) -> Self;
    fn serialize_ack(&self) -> Vec<u8>;
    fn deserialize_ack(&mut self, data: &[u8]);
}

pub const STANDARD_CONFIG: bincode::config::Configuration<LittleEndian, Fixint> = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .with_no_limit();

// A helper to auto-derive serialization/deserialization
#[macro_export]
macro_rules! handshake_protocol {
    (
        protocol $protocol_name:ident {
            $(
                handshake $handshake_name:ident {
                    req: $req_ty:ty,
                    ack: $ack_ty:ty
                }
            ),+ $(,)?
        }
    ) => {
        
        #[derive(Debug, Serialize, Encode, Decode, Deserialize, Clone)]
        pub enum $protocol_name {
            $(
                $handshake_name {
                    req: $req_ty,
                    ack: Option<$ack_ty>,
                }
            ),+
        }
        
        impl Default for $protocol_name {
            fn default() -> Self {
                Default::default()
            }
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

        impl HandshakeProtocol for $protocol_name {
            // Serialize request message into bytes
            fn serialize_req(&self) -> Vec<u8> {
                match self {
                    $(
                        $protocol_name::$handshake_name { req, .. } => {
                            bincode::encode_to_vec(req, STANDARD_CONFIG).unwrap()
                        }
                    ),+
                }
            }

            // Deserialize request message from bytes
            fn deserialize_req(handshake: &str, data: &[u8]) -> Self {
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

            // Serialize response message into bytes
            fn serialize_ack(&self) -> Vec<u8> {
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
            fn deserialize_ack(&mut self, data: &[u8]) {
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