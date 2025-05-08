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
        protocol $nested_protocol_name:ident {
            $($nested_body:tt)*
        }
        $($rest:tt)*
    ) => {
        handshake_protocol!(@protocol $nested_protocol_name, [$($path)* $protocol_name], $($nested_body)*);
        handshake_protocol!(@protocol $protocol_name, [$($path)*], $($rest)*);
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
        //handshake_protocol!(@protocol $protocol_name, [$($path)*], $($rest)*);
    };
    (@protocol $protocol_name:ident, [$($path:ident)*],) => {
        
    }
}