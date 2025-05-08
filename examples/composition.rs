use serde::{Serialize, Deserialize};
use core::task::Poll;
pub use protocol_builder::{
    A64,
    handshake_protocol,
    HandshakeProtocol,
    RequestBuilder,
    Encode,
    Decode,
    STANDARD_CONFIG,
};

// Example message types (must implement Serialize/Deserialize + Default)
#[derive(Clone, Serialize, Deserialize, Encode, Decode, Debug, Default)]
pub struct NonceCommit {
    pub nonce_hash: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize, Encode, Decode, Debug)]
struct PartialSig {
    #[serde( with = "A64")]
    sig_part: [u8; 64],
}

impl Default for PartialSig {
    fn default() -> Self {
        Self {
            sig_part: [0; 64]
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Encode, Decode, Debug)]
struct AggSig {
    #[serde( with = "A64")]
    aggregated_sig: [u8; 64],
}

impl Default for AggSig {
    fn default() -> Self {
        Self {
            aggregated_sig: [0; 64]
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Encode, Decode, Debug, Default)]
pub enum HandshakeFinality {
    Accept,
    #[default]
    Reject,
}

#[derive(Clone, Serialize, Deserialize, Encode, Decode, Debug, Default)]
pub enum Empty {
    #[default]
    Alphabet,
    Lexicon,
} // For requests without payload

// Applying the macro explicitly to MuSig2 protocol
handshake_protocol! {
    protocol MuSig2Protocol {
        handshake NonceCommitment {
            req: Empty,                // Coordinator request: no payload
            ack: NonceCommit,          // Participants response
        }
        handshake NonceReveal {
            req: NonceCommit,                // Coordinator request: NonceCommit
            ack: ParSigProtocol,          // Participants response
        }
        handshake PartialSignature {
            req: ParSigProtocol,                // Coordinator request: NonceReveal
            ack: PartialSig,           // Participants response
        }
        handshake AggregateSignature {
            req: PartialSig,                // Verifier request: PartialSig
            ack: AggSig,               // Coordinator final aggregated response
        }
        handshake Finality {
            req: AggSig,
            ack: HandshakeFinality,
        }
    }
}

impl MuSig2Protocol {
    fn new() -> Self {
        Self::NonceCommitment {
            req: Default::default(),
            ack: None,
        }
    }
    fn init(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let empty = Self::new();
        let first_handshake = &Self::list_handshakes()[0];
        Ok(Self::req_decode(first_handshake, data))
    }
    fn finalize(&self) -> Poll<&HandshakeFinality> {
        match self {
            MuSig2Protocol::Finality { ack: Some(ack), .. } => Poll::from(ack),
            _ => Poll::Pending,
        }
    }
}

// Example usage:
fn main() {
    // Coordinator initiates a handshake requesting nonce commitments:
    let mut handshake = MuSig2Protocol::NonceCommitment {
        req: Empty::Lexicon,
        ack: None, // initially empty
    };

    // Serialize request (empty payload here, but could have content):
    let serialized_request = handshake.req_encode();
    
    println!("Original deserialized request: {handshake:?}");
    println!("Original serialized request: {serialized_request:?}");
    
    let reinitialized = MuSig2Protocol::init(&serialized_request).unwrap();
    println!("Original reinitialized {:?}", reinitialized);
    
    let handshake_variant = MuSig2Protocol::NonceCommitment {
        req: Empty::Alphabet,
        ack: None, // initially empty
    };
    println!("Handshake variant : {:?} {:?}", handshake_variant, handshake_variant.req_encode());

    // Participant receives the request, deserializes:
    let mut received_handshake = MuSig2Protocol::req_decode("NonceCommitment", &serialized_request);
    
    println!("Initialized Handshake : {:?}", received_handshake);

    // Participant generates response:
    received_handshake = MuSig2Protocol::NonceCommitment {
        req: Empty::Alphabet,
        ack: Some(NonceCommit {
            nonce_hash: *b"a long nonce hash to place into.",
        }),
    };
    
    let aggregate_signature = MuSig2Protocol::AggregateSignature {
        req: PartialSig::default(),
        ack: Some(AggSig {
            aggregated_sig: *b"a long nonce hash to place into, after some rare events show up.",
        }),        
    };
    
    println!("Serialized Aggregate Signature Acknowledge : {:?}", aggregate_signature.ack_encode());

    // Participant serializes the response:
    let serialized_ack = received_handshake.ack_encode();
    
    println!("Serialized ACK : {serialized_ack:?}");

    // Coordinator deserializes the response:
    handshake.ack_decode(&serialized_ack);

    println!("Handshake State: {:?}", handshake);
    println!("Coordinator received: {:?}", handshake.ack_encode());
    
    println!("List protocol: {:?}", MuSig2Protocol::list_protocol_types());
    println!("List handhshakes: {:?}", MuSig2Protocol::list_handshakes());
}