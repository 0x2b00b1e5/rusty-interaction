use ed25519_dalek::Verifier;
use ed25519_dalek::{PublicKey, Signature};
use std::convert::TryInto;
/// If verification fails, it will return the `ValidationError` enum.
pub enum ValidationError {
    /// For anything related to conversion errors
    KeyConversionError {
        /// What error?
        name: &'static str,
    },
    /// For invalid keys
    InvalidSignatureError,
}

/// Verifies an incoming Interaction.
/// This verification is mandatory for every incoming Interaction.
/// See [the developer docs](https://discord.com/developers/docs/interactions/slash-commands#security-and-authorization) for more info
pub fn verify_discord_message(
    public_key: PublicKey,
    signature: &str,
    timestamp: &str,
    body: &str,
) -> Result<(), ValidationError> {
    let signature_bytes = hex::decode(signature)
        .map_err(|_| ValidationError::KeyConversionError { name: "Hex conversion" })?;

    let signature_bytes: [u8; 64] = signature_bytes.try_into()
        .map_err(|_| ValidationError::KeyConversionError {
                name: "Signature Length",
        })?;

    let signature = Signature::new(signature_bytes);

    // Format the data to verify (Timestamp + body)
    let msg = format!("{}{}", timestamp, body);

    public_key
        .verify(msg.as_bytes(), &signature)
        .map_err(|_| ValidationError::InvalidSignatureError)
}
