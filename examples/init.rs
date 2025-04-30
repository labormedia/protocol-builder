use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode};
pub use protocol_builder::{
    A64,
    handshake_protocol,
    HandshakeProtocol,
    RequestBuilder,
    STANDARD_CONFIG,
};

// Example message types (must implement Serialize/Deserialize + Default)
#[derive(Clone, Serialize, Deserialize, Encode, Decode, Debug, Default)]
pub struct NonceCommit {
    pub nonce_hash: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize, Encode, Decode, Debug, Default)]
struct NonceReveal {
    nonce: [u8; 32],
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
            ack: NonceCommit          // Participants response
        },
        handshake NonceReveal {
            req: NonceCommit,                // Coordinator request: NonceCommit
            ack: NonceReveal          // Participants response
        },
        handshake PartialSignature {
            req: NonceReveal,                // Coordinator request: NonceReveal
            ack: PartialSig           // Participants response
        },
        handshake AggregateSignature {
            req: PartialSig,                // Verifier request: PartialSig
            ack: AggSig               // Coordinator final aggregated response
        },
    }
}

// Example usage:
fn main() {
    // Coordinator initiates a handshake requesting nonce commitments:
    let handshake = MuSig2Protocol::NonceCommitment {
        req: Empty::Lexicon,
        ack: Default::default(), // initially empty
    };

    // Serialize request (empty payload here, but could have content):
    let serialized_request = handshake.serialize_req();

    // Participant receives the request, deserializes:
    let mut received_handshake = MuSig2Protocol::deserialize_req("NonceCommitment", &serialized_request);

    // Participant generates response:
    received_handshake = MuSig2Protocol::NonceCommitment {
        req: Empty::Alphabet,
        ack: Some(NonceCommit {
            nonce_hash: *b"a long nonce hash to place into.",
        }),
    };

    // Participant serializes the response:
    let serialized_ack = received_handshake.serialize_ack();
    
    println!("Input : {:?}", received_handshake);
    println!("Serialized ACK : {serialized_ack:?} {:?}", serialized_ack.len());

    // Coordinator deserializes the response:
    let mut coordinator_handshake = handshake; // original handshake
    coordinator_handshake.deserialize_ack(&serialized_ack);

    println!("Coordinator received: {:?}", coordinator_handshake);
}