use keyring::Entry;
use zeroize::Zeroizing;

const SERVICE: &str = "college-wifi";
const USERNAME_ACCOUNT: &str = "username";
const PASSWORD_ACCOUNT: &str = "password";

pub struct Credentials {
    pub username: String,
    pub password: Zeroizing<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("credentials are not configured; run `wifi credentials set`")]
    Missing,
    #[error("the operating system credential store failed: {0}")]
    Store(#[from] keyring::Error),
}

fn entry(account: &str) -> Result<Entry, CredentialError> {
    #[cfg(target_os = "linux")]
    let entry = Entry::new_with_target(SERVICE, SERVICE, account);
    #[cfg(not(target_os = "linux"))]
    let entry = Entry::new(SERVICE, account);

    entry.map_err(CredentialError::Store)
}

pub fn load_credentials() -> Result<Credentials, CredentialError> {
    let username = entry(USERNAME_ACCOUNT)?
        .get_password()
        .map_err(|error| match error {
            keyring::Error::NoEntry => CredentialError::Missing,
            other => CredentialError::Store(other),
        })?;
    let password = entry(PASSWORD_ACCOUNT)?
        .get_password()
        .map_err(|error| match error {
            keyring::Error::NoEntry => CredentialError::Missing,
            other => CredentialError::Store(other),
        })?;

    Ok(Credentials {
        username,
        password: Zeroizing::new(password),
    })
}

pub fn save_credentials(username: &str, password: &str) -> Result<(), CredentialError> {
    entry(USERNAME_ACCOUNT)?.set_password(username)?;
    entry(PASSWORD_ACCOUNT)?.set_password(password)?;
    Ok(())
}
