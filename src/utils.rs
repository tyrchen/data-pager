use crate::error::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use snafu::ResultExt;

#[inline(always)]
pub(crate) fn b64_encode(input: impl AsRef<[u8]>) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

#[inline(always)]
pub(crate) fn b64_decode(s: &str, output: &mut [u8]) -> Result<usize> {
    URL_SAFE_NO_PAD
        .decode_slice(s, output)
        .context(Base64DecodeSnafu { s })
}

/// encode u64 to base64 string
#[inline(always)]
pub(crate) fn encode_u64(input: u64) -> String {
    let s = input.to_string();
    b64_encode(s)
}

/// decode base64 string to u64
#[inline(always)]
pub(crate) fn decode_u64(s: &str) -> Result<u64> {
    let mut bytes = [0u8; 32];
    let len = b64_decode(s, &mut bytes)?;
    let s = std::str::from_utf8(&bytes[..len]).context(InvalidUtf8Snafu)?;
    s.parse::<u64>().context(InvalidNumberSnafu { s })
}
