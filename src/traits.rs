use core::pin::Pin;
use serde::{Serializer, Deserializer};

pub trait BigArray<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>;
}

pub trait RequestBuilder {
    fn req_decode(handshake: &str, data: &[u8]) -> Self;
}

pub trait HandshakeProtocol {
    fn req_encode(&self) -> Vec<u8>;
    fn ack_encode(&self) -> Vec<u8>;
    fn ack_decode(&mut self, data: &[u8]);
}

pub trait AsyncExecutor: HandshakeProtocol {
    fn exec(&mut self) -> Pin<Box<impl Future<Output = Self> + Send>>;
}

pub trait HandshakeGetter {
    type Req;
    type Ack;
    fn get_req_ref(&self) -> &Self::Req;
    fn get_ack_ref(&self) -> &Option<Self::Ack>;
}