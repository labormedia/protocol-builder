use core::pin::Pin;
use serde::{Serializer, Deserializer};

pub trait BigArray<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>;
}

pub trait RequestBuilder {
    fn deserialize_req(handshake: &str, data: &[u8]) -> Self;
}

pub trait HandshakeProtocol {
    fn serialize_req(&self) -> Vec<u8>;
    fn serialize_ack(&self) -> Vec<u8>;
    fn deserialize_ack(&mut self, data: &[u8]);
}

pub trait AsyncExecutor {
    fn exec(&mut self) -> Pin<Box<impl Future<Output = Self> + Send>>;
}