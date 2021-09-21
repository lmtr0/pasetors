use crate::errors::Errors;
use alloc::string::String;
use alloc::vec::Vec;
use ct_codecs::{Base64UrlSafeNoPadding, Decoder, Encoder};

/// Encode bytes with Base64 URL-safe and no padding.
pub(crate) fn encode_b64<T: AsRef<[u8]>>(bytes: T) -> Result<String, Errors> {
    let inlen = bytes.as_ref().len();
    let mut buf = vec![0u8; Base64UrlSafeNoPadding::encoded_len(inlen)?];

    let ret: String = Base64UrlSafeNoPadding::encode_to_str(&mut buf, bytes)?.into();

    Ok(ret)
}

/// Decode string with Base64 URL-safe and no padding.
pub(crate) fn decode_b64<T: AsRef<[u8]>>(encoded: T) -> Result<Vec<u8>, Errors> {
    let inlen = encoded.as_ref().len();
    // We can use encoded len here, even if it returns more than needed,
    // because ct-codecs allows this.
    let mut buf = vec![0u8; Base64UrlSafeNoPadding::encoded_len(inlen)?];

    let ret: Vec<u8> = Base64UrlSafeNoPadding::decode(&mut buf, encoded, None)?.into();

    Ok(ret)
}

/// Validate that a token begins with a given header.purpose and does not contain more than:
/// header.purpose.payload.footer
/// If a footer is present, this is validated against the supplied.
pub(crate) fn validate_format_footer<'a>(
    header: &'a str,
    token: &'a str,
    footer: &[u8],
) -> Result<Vec<&'a str>, Errors> {
    use orion::util::secure_cmp;

    if !token.starts_with(header) {
        return Err(Errors::TokenFormatError);
    }

    let parts_split = token.split('.').collect::<Vec<&str>>();
    if parts_split.len() < 3 || parts_split.len() > 4 {
        return Err(Errors::TokenFormatError);
    }

    let is_footer_present = parts_split.len() == 4;
    if !is_footer_present && !footer.is_empty() {
        return Err(Errors::TokenValidationError);
    }
    if is_footer_present {
        if footer.is_empty() {
            return Err(Errors::TokenValidationError);
        }

        let token_footer = decode_b64(parts_split[3])?;
        if secure_cmp(footer, token_footer.as_ref()).is_err() {
            return Err(Errors::TokenValidationError);
        }
    }

    Ok(parts_split)
}

#[cfg(test)]
pub(crate) mod tests {
    use alloc::string::String;
    use alloc::vec::Vec;
    use serde::{Deserialize, Serialize};

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct TestFile {
        pub(crate) name: String,
        pub(crate) tests: Vec<PasetoTest>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct PasetoTest {
        pub(crate) name: String,
        #[serde(rename(deserialize = "expect-fail"))]
        pub(crate) expect_fail: bool,
        pub(crate) key: Option<String>,
        pub(crate) nonce: Option<String>,
        #[serde(rename(deserialize = "public-key"))]
        pub(crate) public_key: Option<String>,
        #[serde(rename(deserialize = "secret-key"))]
        pub(crate) secret_key: Option<String>,
        #[serde(rename(deserialize = "public-key-pem"))]
        pub(crate) public_key_pem: Option<String>,
        #[serde(rename(deserialize = "secret-key-pem"))]
        pub(crate) secret_key_pem: Option<String>,
        pub(crate) token: String,
        pub(crate) payload: Option<Payload>,
        pub(crate) footer: String,
        #[serde(rename(deserialize = "implicit-assertion"))]
        pub(crate) implicit_assertion: String,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct Payload {
        pub(crate) data: String,
        pub(crate) exp: String,
    }
}
