use crate::daemon::DaemonConfig;
use crate::ssh_protocol::{parse_message, SshMessage};
use anyhow::Result;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use tracing::{debug, error, info};
use uuid::Uuid;

#[cfg(unix)]
use tokio::net::UnixStream;

#[cfg(unix)]
pub async fn handle_client(
    mut socket: tokio::net::UnixStream,
    config: DaemonConfig,
) -> Result<()> {
    let mut buffer = vec![0u8; 4096];

    // Read the client request
    let n = socket.read(&mut buffer).await?;
    if n == 0 {
        return Ok(());
    }

    let request_data = &buffer[..n];
    let message = parse_message(request_data)?;

    debug!("Received SSH agent message: {:?}", message);

    if message.is_sign_request() {
        // This is a signature request - needs approval
        let request_id = Uuid::new_v4().to_string();

        match &message {
            SshMessage::SignRequest { data, .. } => {
                // Create a human-readable representation
                let hex_preview = hex::encode(&data[..std::cmp::min(32, data.len())]);
                let command = format!("SSH Sign Request (data: {}...)", hex_preview);

                info!("Sign request received: {}", request_id);

                // Send approval request to Telegram
                crate::telegram::send_approval_request(
                    &config.bot_token,
                    config.chat_id,
                    &request_id,
                    &command,
                )
                .await?;

                // Wait for approval (with timeout)
                // Note: In a real implementation, you'd need a channel to receive approvals
                // from an update handler that polls Telegram Bot API
                let wait_duration = Duration::from_secs(config.approval_timeout_secs);
                match timeout(wait_duration, wait_for_approval(&request_id)).await {
                    Ok(Ok(approved)) => {
                        if approved {
                            // Forward to real SSH agent
                            forward_to_ssh_agent(
                                &config,
                                request_data,
                                &mut socket,
                            )
                            .await?;
                        } else {
                            // Send failure response
                            send_failure(&mut socket).await?;
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Approval error: {}", e);
                        send_failure(&mut socket).await?;
                    }
                    Err(_) => {
                        info!("Approval timeout for request: {}", request_id);
                        send_failure(&mut socket).await?;
                    }
                }
            }
            _ => unreachable!(),
        }
    } else {
        // Non-signing requests are forwarded directly
        forward_to_ssh_agent(&config, request_data, &mut socket).await?;
    }

    Ok(())
}

#[cfg(unix)]
async fn forward_to_ssh_agent(
    config: &DaemonConfig,
    request: &[u8],
    client_socket: &mut tokio::net::UnixStream,
) -> Result<()> {
    let ssh_auth_sock = config
        .ssh_auth_sock
        .clone()
        .or_else(|| std::env::var("SSH_AUTH_SOCK").ok())
        .ok_or_else(|| anyhow::anyhow!("SSH_AUTH_SOCK not set"))?;

    let mut agent_socket = UnixStream::connect(&ssh_auth_sock).await?;
    agent_socket.write_all(request).await?;

    let mut buffer = vec![0u8; 8192];
    let n = agent_socket.read(&mut buffer).await?;

    client_socket.write_all(&buffer[..n]).await?;
    debug!("Response forwarded to client");

    Ok(())
}

async fn wait_for_approval(_request_id: &str) -> Result<bool> {
    // TODO: Implement proper approval mechanism
    // This would involve:
    // 1. Storing pending requests in a shared map
    // 2. Running a separate task that polls Telegram updates
    // 3. Setting approval status when callback is received

    // For now, just return a placeholder
    tokio::time::sleep(Duration::from_secs(5)).await;
    Ok(true)
}

async fn send_failure(socket: &mut tokio::net::UnixStream) -> Result<()> {
    let response = [0x00, 0x00, 0x00, 0x01, 0x05]; // length=1, type=SSH_AGENT_FAILURE
    socket.write_all(&response).await?;
    Ok(())
}
