use anyhow::Result;
use std::collections::HashMap;
use std::io::BufRead;
use tracing::debug;

use crate::biometric;
use crate::keyring_mgr;

/// Credential attributes from git credential protocol
#[derive(Debug, Clone)]
pub struct CredentialRequest {
    pub protocol: String,
    pub host: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Parse Git credential protocol request from stdin
pub fn parse_credential_request(reader: impl BufRead) -> Result<CredentialRequest> {
    let mut attrs = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }

        if let Some((key, value)) = line.split_once('=') {
            attrs.insert(key.to_string(), value.to_string());
        }
    }

    Ok(CredentialRequest {
        protocol: attrs.get("protocol").cloned().unwrap_or_default(),
        host: attrs.get("host").cloned().unwrap_or_default(),
        username: attrs.get("username").cloned(),
        password: attrs.get("password").cloned(),
    })
}

pub async fn handle_get(req: &CredentialRequest) -> Result<()> {
    debug!("Getting credentials for: {}://{}", req.protocol, req.host);

    // Step 1: Verify user presence with biometric
    biometric::verify_user_presence().await?;

    // Step 2: Retrieve from secure storage
    let credential_key = format!("{}://{}", req.protocol, req.host);
    let password = keyring_mgr::get_password(&credential_key)?;

    // Step 3: Output in git credential format
    if let Some(pwd) = password {
        println!("username={}", req.username.as_deref().unwrap_or(""));
        println!("password={}", pwd);
    }

    Ok(())
}

pub async fn handle_approve(req: &CredentialRequest) -> Result<()> {
    debug!("Storing credentials for: {}://{}", req.protocol, req.host);

    if let Some(ref password) = req.password {
        let credential_key = format!("{}://{}", req.protocol, req.host);
        keyring_mgr::set_password(&credential_key, password)?;
    }

    Ok(())
}

pub async fn handle_reject(req: &CredentialRequest) -> Result<()> {
    debug!("Rejecting/clearing credentials for: {}://{}", req.protocol, req.host);

    let credential_key = format!("{}://{}", req.protocol, req.host);
    keyring_mgr::delete_password(&credential_key)?;

    Ok(())
}
