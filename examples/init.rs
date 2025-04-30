use serde::{Serialize, Deserialize};
pub use protocol_builder::{
    A64,
    handshake_protocol,
};

// Example message types (must implement Serialize/Deserialize + Default)
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NonceCommit {
    pub nonce_hash: [u8; 32],
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct NonceReveal {
    nonce: [u8; 32],
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum Empty {
    #[default]
    Alphabet,
    Lexicon,
} // For requests without payload

// Applying the macro explicitly to MuSig2 protocol
handshake_protocol! {
    protocol MuSig2Protocol {
        handshake NonceCommitment {
            request: Empty,                // Coordinator request: no payload
            response: NonceCommit          // Participants response
        },
        handshake NonceReveal {
            request: NonceCommit,                // Coordinator request: NonceCommit
            response: NonceReveal          // Participants response
        },
        handshake PartialSignature {
            request: NonceReveal,                // Coordinator request: NonceReveal
            response: PartialSig           // Participants response
        },
        handshake AggregateSignature {
            request: PartialSig,                // Verifier request: PartialSig
            response: AggSig               // Coordinator final aggregated response
        },
    }
}

// Example usage:
fn main() {
    // Coordinator initiates a handshake requesting nonce commitments:
    let handshake = MuSig2Protocol::NonceCommitment {
        request: Empty::Lexicon,
        response: Default::default(), // initially empty
    };

    // Serialize request (empty payload here, but could have content):
    let serialized_request = handshake.serialize_request();

    // Participant receives the request, deserializes:
    let mut received_handshake = MuSig2Protocol::deserialize_request("NonceCommitment", &serialized_request);

    // Participant generates response:
    received_handshake = MuSig2Protocol::NonceCommitment {
        request: Empty::Alphabet,
        response: NonceCommit {
            nonce_hash: [42;32],
        },
    };

    // Participant serializes the response:
    let serialized_response = received_handshake.serialize_response();
    
    println!("Serialized response : {serialized_response:?}");

    // Coordinator deserializes the response:
    let mut coordinator_handshake = handshake; // original handshake
    coordinator_handshake.deserialize_response(&serialized_response);

    println!("Coordinator received: {:?}", coordinator_handshake);
}