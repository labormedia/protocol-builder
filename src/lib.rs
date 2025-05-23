mod traits;
mod macros;
pub use traits::{
    BigArray,
    RequestBuilder,
    HandshakeProtocol,
    AsyncExecutor,
    HandshakeGetter,
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

