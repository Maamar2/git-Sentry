# Git-Sentry: SSH Agent Proxy with Out-of-Band 2FA

A Rust daemon that intercepts SSH signing requests and requires Telegram approval before allowing commits or pushes.

## Quick Start

### 1. Setup Telegram Bot

```bash
# Contact @BotFather on Telegram
# Create a new bot and save the token

# Get your chat ID by sending a message to the bot, then:
curl https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates
# Find the "id" field in the response
```

### 2. Build & Install

```bash
cargo build --release
sudo cp target/release/git-sentry /usr/local/bin/
```

### 3. Configure

```bash
# Set environment variables
export GIT_SENTRY_BOT_TOKEN="your_bot_token"
export GIT_SENTRY_CHAT_ID=1234567890

# Point SSH_AUTH_SOCK to Git-Sentry
export SSH_AUTH_SOCK="/tmp/git-sentry.sock"
```

### 4. Start Daemon

```bash
# Test Telegram connection first
git-sentry test --bot-token "$GIT_SENTRY_BOT_TOKEN" --chat-id $GIT_SENTRY_CHAT_ID

# Start the daemon
git-sentry daemon --bot-token "$GIT_SENTRY_BOT_TOKEN" --chat-id $GIT_SENTRY_CHAT_ID
```

## How It Works

1. Git requests a signature from the SSH agent
2. Git-Sentry intercepts the request
3. A Telegram notification is sent to your phone
4. You approve or deny the operation
5. If approved, request is forwarded to your real SSH agent
6. Signature is returned to Git
7. If denied/timeout, Git receives an error

## Protocol Details

Git-Sentry implements the SSH Agent Protocol (RFC draft-miller-ssh-agent):

- Listens on a Unix socket
- Parses binary SSH agent protocol messages
- Detects signature requests (`SSH_AGENTC_SIGN_REQUEST`)
- Forwards other requests transparently

## Architecture

```
Git -> Git-Sentry Socket -> [Telegram Approval] -> Real SSH Agent -> Git
                           (blocks until approved)
```

## Configuration Options

```bash
git-sentry daemon \
  --socket /tmp/git-sentry.sock \           # Unix socket path
  --bot-token "YOUR_TOKEN" \                # Telegram bot token
  --chat-id 1234567890 \                    # Telegram chat ID
  --ssh-auth-sock /run/user/1000/ssh.sock \ # Real SSH agent socket
  --timeout 300                              # Approval timeout in seconds
```

## Environment Variables

```bash
GIT_SENTRY_BOT_TOKEN      # Telegram bot token
GIT_SENTRY_CHAT_ID        # Telegram chat ID
GIT_SENTRY_SOCKET         # Custom socket path
SSH_AUTH_SOCK             # Real SSH agent socket
```

## Security Notes

⚠️ **This is the first layer of defense**, not a complete security solution.

**Protected against:**
- Malware trying to forge commits on your behalf
- Rogue scripts using your SSH key
- Unauthorized key usage

**Does NOT protect against:**
- Physical access to your machine (after daemon starts)
- Compromised SSH keys (this tool doesn't protect the key itself)
- OS-level exploits (malware with root access)

## Systemd Integration

Create `/etc/systemd/user/git-sentry.service`:

```ini
[Unit]
Description=Git-Sentry SSH Agent Proxy
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/git-sentry daemon \
  --bot-token %E/git-sentry-token \
  --chat-id 1234567890
Environment="GIT_SENTRY_BOT_TOKEN=your_token"
Environment="GIT_SENTRY_CHAT_ID=1234567890"
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
```

Then:

```bash
systemctl --user enable git-sentry
systemctl --user start git-sentry
```

## Troubleshooting

### Socket already in use
```bash
rm /tmp/git-sentry.sock
# Restart the daemon
```

### SSH_AUTH_SOCK not recognized
```bash
# Make sure it points to Git-Sentry socket
export SSH_AUTH_SOCK="/tmp/git-sentry.sock"

# Verify:
ls -la $SSH_AUTH_SOCK
```

### Telegram test fails
```bash
# Check token and chat ID
git-sentry test --bot-token "token" --chat-id 12345

# Verify network connectivity
curl https://api.telegram.org/bot<TOKEN>/getMe
```

### No approval requests appear
```bash
# Check if requests are being forwarded
sudo strace -f -p $(pgrep git-sentry)

# Verify SSH keys are loaded
ssh-add -l

# Check if git is actually using your SSH_AUTH_SOCK
SSH_AUTH_SOCK=/tmp/git-sentry.sock git clone git@github.com:user/repo.git
```

## Development

```bash
# Run tests
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

## Implementation Notes

### SSH Agent Protocol

The SSH Agent Protocol uses a simple binary format:

```
[4 bytes: message length][1 byte: message type][variable: payload]
```

Message types:
- `11` (0x0B): `SSH_AGENTC_REQUEST_IDENTITIES` - Get available keys
- `13` (0x0D): `SSH_AGENTC_SIGN_REQUEST` - Sign data (intercepted)
- `5` (0x05): `SSH_AGENT_FAILURE` - Operation failed
- `14` (0x0E): `SSH_AGENT_SIGN_RESPONSE` - Signature result

### Approval Flow

```
[User sends request]
        ↓
   [Parse message]
        ↓
  [Is it signing?] → No → [Forward directly]
        ↓
       Yes
        ↓
 [Generate request ID]
        ↓
 [Send Telegram notification]
        ↓
[Wait for approval (with timeout)]
        ↓
 [Approval received?] → No → [Send failure response]
        ↓
       Yes
        ↓
[Forward to real SSH agent]
        ↓
[Return signature to client]
```

## Future Enhancements

- [ ] Windows Named Pipes support
- [ ] Hardware security key (YubiKey) integration
- [ ] Request analytics and audit logging
- [ ] Per-command approval policies
- [ ] Slack/Discord notification support
- [ ] SQLite request history database
- [ ] Web dashboard for approvals

## Related Projects

- **Bio-Git**: Biometric credential verification for HTTPS
- See parent directory `README.md` for more details

## License

MIT or Apache-2.0
