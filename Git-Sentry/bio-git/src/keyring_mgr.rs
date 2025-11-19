use anyhow::Result;
use keyring::Entry;
use tracing::{debug, info};

const KEYRING_SERVICE: &str = "bio-git";

/// Store password in secure keyring
pub fn set_password(key: &str, password: &str) -> Result<()> {
    debug!("Storing credential for: {}", key);

    let entry = Entry::new(KEYRING_SERVICE, key)?;
    entry.set_password(password)?;

    info!("✓ Credential stored securely");
    Ok(())
}

/// Retrieve password from secure keyring
pub fn get_password(key: &str) -> Result<Option<String>> {
    debug!("Retrieving credential for: {}", key);

    let entry = Entry::new(KEYRING_SERVICE, key)?;
    match entry.get_password() {
        Ok(password) => {
            info!("✓ Credential retrieved from keyring");
            Ok(Some(password))
        }
        Err(keyring::Error::NoEntry) => {
            debug!("No credential found for: {}", key);
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// Delete password from keyring
pub fn delete_password(key: &str) -> Result<()> {
    debug!("Deleting credential for: {}", key);

    let entry = Entry::new(KEYRING_SERVICE, key)?;
    match entry.delete_password() {
        Ok(_) => {
            info!("✓ Credential deleted");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            debug!("No credential found to delete");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
