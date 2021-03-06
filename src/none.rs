//! [`Verifier`] and [`Signer`] implementations for the `none` algorithm.
//!
//! The `none` algorithm is defined in [RFC 7518 section 3.6](https://tools.ietf.org/html/rfc7518#section-3.6).
//! It does not provide any integrity protection.
//!
//! It doesn't often make sense to use this "algorithm".

use crate::{Error, JsonObject, JsonValue, parse_required_header_param, Result, Signer, Verifier};

/// Message verifier for the `none` algorithm.
///
/// The `none` algorithm has an empty signature and does not provide integrity protection.
/// The verifier does check that the signature is indeed empty as required by [RFC 7518 (section 3.6)](https://tools.ietf.org/html/rfc7518#section-3.6).
#[derive(Copy, Clone, Debug)]
pub struct NoneVerifier;

/// Message signer for the `none` algorithm.
///
/// Adds an empty signature that does not provide integrity protection.
#[derive(Copy, Clone, Debug)]
pub struct NoneSigner;

impl Verifier for NoneVerifier {
	fn verify(&self, protected_header: Option<&JsonObject>, unprotected_header: Option<&JsonObject>, _encoded_header: &[u8], _encoded_payload: &[u8], signature: &[u8]) -> Result<()> {
		let algorithm : &str = parse_required_header_param(protected_header, unprotected_header, "alg")?;

		if algorithm != "none" {
			Err(Error::unsupported_mac_algorithm(algorithm))
		} else if !signature.is_empty() {
			Err(Error::invalid_signature(""))
		} else {
			Ok(())
		}
	}
}

impl Signer for NoneSigner {
	fn set_header_params(&self, header: &mut JsonObject) {
		header.insert("alg".to_string(), JsonValue::from("none"));
	}

	fn compute_mac(&self, _encoded_header: &[u8], _encoded_payload: &[u8]) -> Result<Vec<u8>> {
		Ok(Vec::new())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{ErrorKind, json_object};
	use assert2::assert;

	#[test]
	fn test_none_signer_header() {
		let mut header = json_object!{};
		let signer = NoneSigner;

		signer.set_header_params(&mut header);
		assert!(header == json_object!{"alg": "none"});
	}

	#[test]
	fn test_none_signer_mac() {
		let signer = NoneSigner;
		assert!(&signer.compute_mac(b"fake_header", b"fake_payload").unwrap() == b"");
		assert!(&signer.compute_mac(b"fake_header", b"").unwrap() == b"");
		assert!(&signer.compute_mac(b"",            b"fake_payload").unwrap() == b"");
		assert!(&signer.compute_mac(b"",            b"").unwrap() == b"");
	}

	#[test]
	fn test_verify_none() {
		let header  = &json_object!{"alg": "none"};
		let verifier = NoneVerifier;

		// Test that an empty signature is accepted.
		assert!(let Ok(_) = verifier.verify(Some(header), None, b"fake_header", b"fake_payload", b""));
		assert!(let Ok(_) = verifier.verify(Some(header), None, b"fake_header", b"",             b""));
		assert!(let Ok(_) = verifier.verify(Some(header), None, b"",            b"fake_payload", b""));
		assert!(let Ok(_) = verifier.verify(Some(header), None, b"",            b"fake_payload", b""));

		// Test that a non-empty signature is rejected.
		assert!(let Err(Error { kind: ErrorKind::InvalidSignature, .. }) = verifier.verify(Some(header), None, b"fake_header", b"fake_payload", b"bad-signature"));
		assert!(let Err(Error { kind: ErrorKind::InvalidSignature, .. }) = verifier.verify(Some(header), None, b"fake_header", b"",             b"bad-signature"));
		assert!(let Err(Error { kind: ErrorKind::InvalidSignature, .. }) = verifier.verify(Some(header), None, b"",            b"fake_payload", b"bad-signature"));
		assert!(let Err(Error { kind: ErrorKind::InvalidSignature, .. }) = verifier.verify(Some(header), None, b"",            b"fake_payload", b"bad-signature"));
	}
}
