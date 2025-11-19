use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct DaemonConfig {
    pub socket_path: Option<String>,
    pub bot_token: String,
    pub chat_id: i64,
    pub ssh_auth_sock: Option<String>,
    pub approval_timeout_secs: u64,
}

impl DaemonConfig {
    pub fn socket_path(&self) -> PathBuf {
        let path = self.socket_path.clone().unwrap_or_else(|| {
            #[cfg(unix)]
            {
                "/tmp/git-sentry.sock".to_string()
            }
            #[cfg(windows)]
            {
                std::env::var("TEMP")
                    .map(|temp| format!("{}\\git-sentry.sock", temp))
                    .unwrap_or_else(|_| "git-sentry.sock".to_string())
            }
        });
        PathBuf::from(path)
    }
}

#[cfg(unix)]
pub async fn run(config: DaemonConfig) -> Result<()> {
    use std::os::unix::io::AsRawFd;
    use tokio::net::UnixListener;
    use tracing::info;

    let socket_path = config.socket_path();

    // Clean up old socket if it exists
    let _ = std::fs::remove_file(&socket_path);

    let listener = UnixListener::bind(&socket_path)?;
    info!("Git-Sentry daemon listening on: {:?}", socket_path);

    loop {
        let (socket, _) = listener.accept().await?;
        let config = config.clone();

        tokio::spawn(async move {
            if let Err(e) = crate::proxy::handle_client(socket, config).await {
                tracing::error!("Error handling client: {}", e);
            }
        });
    }
}

#[cfg(windows)]
pub async fn run(_config: DaemonConfig) -> Result<()> {
    anyhow::bail!("Unix sockets are not natively supported on Windows. \
                   Consider using Windows Named Pipes or running on WSL2.");
}
