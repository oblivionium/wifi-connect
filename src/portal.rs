use rand::rngs::OsRng;
use reqwest::blocking::{Client, Response};
use reqwest::header::CONTENT_TYPE;
use rsa::{pkcs1v15::Pkcs1v15Encrypt, BigUint, RsaPublicKey};
use serde::Deserialize;
use std::time::Duration;
use zeroize::Zeroizing;

const PORTAL: &str = "https://172.22.2.6/connect/";

pub struct Portal {
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub enum PortalError {
    #[error("portal request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("portal returned malformed RSA settings")]
    InvalidRsaSettings,
    #[error("portal RSA encryption failed: {0}")]
    Encryption(#[from] rsa::errors::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error(transparent)]
    Portal(#[from] PortalError),
    #[error("portal rejected the credentials ({kind}): {message}")]
    Rejected { kind: String, message: String },
    #[error("portal accepted the login request but did not report an authenticated session")]
    NotAuthenticated,
}

#[derive(Debug, thiserror::Error)]
pub enum LogoutError {
    #[error(transparent)]
    Portal(#[from] PortalError),
    #[error("portal did not confirm that the session ended")]
    StillAuthenticated,
}

#[derive(Deserialize)]
struct RsaSettings {
    m: String,
    e: String,
    #[serde(rename = "loginToken")]
    login_token: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    message: String,
}

#[derive(Deserialize)]
struct Attributes {
    authenticated: bool,
}

impl Portal {
    pub fn new() -> Result<Self, PortalError> {
        let client = Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(12))
            // The portal's web server returns a generic HTTP 500 for reqwest's
            // default user agent. Its normal browser flow succeeds with a
            // browser-style user agent.
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            // The campus portal is served at this private IP with a certificate
            // that does not validate for the IP. The legacy client also ignored
            // certificate errors. This client only sends requests to PORTAL.
            // A hostile network gateway could still impersonate the portal.
            .danger_accept_invalid_certs(true)
            .build()?;
        Ok(Self { client })
    }

    pub fn login(&self, username: &str, password: &str) -> Result<(), LoginError> {
        self.open_portal()?;

        let settings: RsaSettings = self.post_json("RSASettings")?;
        // Captures show the client requesting RSA settings before loading view state.
        let _: serde_json::Value = self.post_json("GetStateAndView")?;
        let encrypted_password = encrypt_password(&settings, password)?;
        let response: LoginResponse = self
            .post_form(
                "Login",
                &[
                    ("realm", "passwordRealm"),
                    ("username", username),
                    ("password", encrypted_password.as_str()),
                ],
            )?
            .json()
            .map_err(PortalError::from)?;

        if response.kind != "SUCCESS" {
            return Err(LoginError::Rejected {
                kind: response.kind,
                message: response.message,
            });
        }

        let _: serde_json::Value = self.post_json("GetStateAndView")?;
        let attributes: Attributes = self.post_json("GetAttributes")?;
        if !attributes.authenticated {
            return Err(LoginError::NotAuthenticated);
        }

        Ok(())
    }

    pub fn logout(&self) -> Result<(), LogoutError> {
        self.open_portal()?;
        let response = self
            .client
            .get(format!("{PORTAL}Logoff"))
            .send()
            .map_err(PortalError::from)?;
        response.error_for_status().map_err(PortalError::from)?;

        let attributes: Attributes = self.post_json("GetAttributes")?;
        if attributes.authenticated {
            return Err(LogoutError::StillAuthenticated);
        }

        Ok(())
    }

    fn open_portal(&self) -> Result<(), PortalError> {
        self.client
            .get(format!("{PORTAL}PortalMain"))
            .send()?
            .error_for_status()?;
        Ok(())
    }

    fn post_json<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T, PortalError> {
        Ok(self.post_empty(endpoint)?.json()?)
    }

    fn post_empty(&self, endpoint: &str) -> Result<Response, PortalError> {
        Ok(self
            .client
            .post(format!("{PORTAL}{endpoint}"))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body("")
            .send()?
            .error_for_status()?)
    }

    fn post_form(&self, endpoint: &str, fields: &[(&str, &str)]) -> Result<Response, PortalError> {
        Ok(self
            .client
            .post(format!("{PORTAL}{endpoint}"))
            .form(fields)
            .send()?
            .error_for_status()?)
    }
}

fn encrypt_password(settings: &RsaSettings, password: &str) -> Result<String, PortalError> {
    let modulus = decode_hex(&settings.m).ok_or(PortalError::InvalidRsaSettings)?;
    let exponent = decode_hex(&settings.e).ok_or(PortalError::InvalidRsaSettings)?;
    let public_key = RsaPublicKey::new(
        BigUint::from_bytes_be(&modulus),
        BigUint::from_bytes_be(&exponent),
    )
    .map_err(|_| PortalError::InvalidRsaSettings)?;

    // The portal encrypts UTF-8(loginToken + password) with randomized PKCS#1 v1.5.
    let mut plaintext = Zeroizing::new(Vec::with_capacity(
        settings.login_token.len() + password.len(),
    ));
    plaintext.extend_from_slice(settings.login_token.as_bytes());
    plaintext.extend_from_slice(password.as_bytes());

    let ciphertext = public_key.encrypt(&mut OsRng, Pkcs1v15Encrypt, &plaintext)?;
    // The portal reverses ciphertext bytes before rendering the hex form value.
    let mut encoded = String::with_capacity(ciphertext.len() * 2);
    for byte in ciphertext.iter().rev() {
        use std::fmt::Write;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    Ok(encoded)
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    let value = value.as_bytes();
    if value.is_empty() {
        return None;
    }
    let mut bytes = Vec::with_capacity((value.len() + 1) / 2);
    let mut index = 0;
    if value.len() % 2 == 1 {
        bytes.push(hex_digit(value[0])?);
        index = 1;
    }
    while index < value.len() {
        bytes.push((hex_digit(value[index])? << 4) | hex_digit(value[index + 1])?);
        index += 2;
    }
    Some(bytes)
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
