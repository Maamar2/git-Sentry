# Git-Sentry & Bio-Git: Secure Git Authentication

Two complementary Rust projects that add out-of-band 2FA and biometric verification to your Git workflows.

## Projects Overview

### 1. **Git-Sentry** - SSH Agent Proxy with Out-of-Band 2FA

A man-in-the-middle SSH Agent that intercepts signing requests and requires Telegram approval before allowing commits or pushes.

**Features:**
- 🔐 Intercepts SSH signing requests
- 📱 Sends approval notifications via Telegram Bot
- ⏱️ Configurable approval timeout
- 🚀 Minimal performance overhead
- ✅ Only allows operations you explicitly approve on your phone

**Use Cases:**
- Protect against malicious scripts or malware attempting to sign commits
- Enforce team-wide signing policies with remote approval
- Audit trail of all signature requests
- Emergency SSH key lockdown from your phone

---

### 2. **Bio-Git** - Git Credential Helper with Biometric Lock

A custom Git credential helper that requires biometric verification (fingerprint/Face ID) for every credential request.

**Features:**
- 👆 Touch ID / Windows Hello / Face ID authentication
- 🔒 Secure keyring-based credential storage
- 📍 Per-repository credential isolation
- ⚡ Sub-second biometric verification
- 🛡️ Prevents credential leakage from background processes

**Use Cases:**
- Require biometric unlock for every Git operation that needs credentials
- Prevent credential exfiltration by malware
- Enforce physical presence authentication
- Seamless biometric integration on macOS and Windows

---

## Project Structure

```
Git-Sentry/
├── git-sentry/          # SSH Agent proxy project
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs      # CLI and daemon startup
│       ├── daemon.rs    # Socket listener and daemon logic
│       ├── ssh_protocol.rs  # SSH Agent Protocol parser
│       ├── telegram.rs   # Telegram Bot API client
│       └── proxy.rs      # SSH request forwarding
│
├── bio-git/             # Git credential helper project
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs      # Git credential protocol handler
│       ├── credential_helper.rs  # Credential protocol parsing
│       ├── biometric.rs  # OS-specific biometric APIs
│       └── keyring_mgr.rs   # Secure credential storage
│
└── README.md
```

---

## Git-Sentry Setup

### Prerequisites
- Rust 1.70+ (for MSRV)
- macOS/Linux (Windows support planned)
- Telegram Bot token

### Installation

```bash
cd git-sentry
cargo build --release
sudo cp target/release/git-sentry /usr/local/bin/
```

### Configuration

1. **Create a Telegram Bot:**
   ```bash
   # Contact @BotFather on Telegram
   # Create a new bot and get your token
   ```

2. **Get your Chat ID:**
   ```bash
   # Send a message to your bot, then:
   curl https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates
   # Look for the "chat" -> "id" value
   ```

3. **Configure Environment Variables:**
   ```bash
   export GIT_SENTRY_BOT_TOKEN="your_bot_token_here"
   export GIT_SENTRY_CHAT_ID=1234567890
   export SSH_AUTH_SOCK="/tmp/git-sentry.sock"  # or your preferred socket path
   ```

4. **Start the Daemon:**
   ```bash
   git-sentry daemon
   ```

   For systemd, create `/etc/systemd/user/git-sentry.service`:
   ```ini
   [Unit]
   Description=Git-Sentry SSH Agent Proxy
   After=network.target
   
   [Service]
   Type=simple
   ExecStart=/usr/local/bin/git-sentry daemon
   Restart=on-failure
   RestartSec=10
   Environment="GIT_SENTRY_BOT_TOKEN=your_token"
   Environment="GIT_SENTRY_CHAT_ID=123456"
   
   [Install]
   WantedBy=default.target
   ```

### Usage

```bash
# Test Telegram connection
git-sentry test --bot-token "your_token" --chat-id 1234567890

# Initialize environment setup
git-sentry setup --bot-token "your_token" --chat-id 1234567890

# Start daemon (runs in foreground, use systemd for background)
git-sentry daemon --bot-token "your_token" --chat-id 1234567890
```

### How It Works

1. Git sends a signing request to the SSH agent
2. Git-Sentry intercepts it and extracts the signature request
3. A notification is sent to your Telegram with "Approve" / "Deny" buttons
4. You approve on your phone within 5 minutes (configurable)
5. If approved, the request is forwarded to the real SSH agent
6. The signature is returned to Git
7. If denied or timeout, Git receives an SSH_AGENT_FAILURE response

---

## Bio-Git Setup

### Prerequisites
- Rust 1.70+
- **Windows:** Windows 10/11 with Hello/Fingerprint configured
- **macOS:** Touch ID capable device
- **Linux:** (Planned) polkit authentication

### Installation

```bash
cd bio-git
cargo build --release
cp target/release/git-credential-bio /usr/local/bin/
```

### Configuration

```bash
# Configure Git to use bio-git credential helper
git config --global credential.helper bio

# Verify it's set
git config --global credential.helper
# Output: bio
```

### Usage

**First time accessing a repository:**
```bash
git clone https://github.com/user/repo.git
# Will prompt: "Bio-Git requires your biometric confirmation"
# Touch fingerprint / Face ID
# Credential is stored and cached securely
```

**Subsequent access to the same repository:**
```bash
git push
# Will request biometric verification again
# This prevents background processes from exfiltrating credentials
```

### How It Works

1. Git needs a credential (e.g., GitHub Personal Access Token)
2. Git calls: `git-credential-bio get`
3. Bio-Git parses the credential request
4. **Biometric verification is requested:**
   - **Windows:** Windows Hello / Fingerprint prompt
   - **macOS:** "Bio-Git wants to access your credentials" + Touch ID
   - **Linux:** (Planned) Polkit authentication
5. User provides biometric (or denies)
6. If verified, credential is retrieved from OS keyring and given to Git
7. If denied, Git receives an error

### Credential Storage Details

Credentials are stored per-host in the OS native keyring:
- **Windows:** Windows Credential Manager
- **macOS:** Keychain
- **Linux:** `secret-service` (libsecret)

Entry format: `bio-git: protocol://host`

---

## Architecture

### Git-Sentry Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Your Terminal                         │
│  $ git push  (requires signature)                            │
└────────────────┬────────────────────────────────────────────┘
                 │
                 │ SSH signature request
                 │
┌────────────────▼────────────────────────────────────────────┐
│         Git-Sentry Daemon (Unix Socket)                      │
│  /tmp/git-sentry.sock                                        │
│                                                               │
│  1. Parse SSH Agent Protocol                                │
│  2. Detect signing request                                  │
│  3. Create approval ID (UUID)                               │
└────────────┬─────────────────────────────┬──────────────────┘
             │                             │
             │ Parse request               │ Timeout: 300s
             │                             │
             │ Extract key info            │
             │                             │
┌────────────▼──────────────────────────┐  │
│   Telegram Bot API                    │  │
│   Send notification with buttons      │  │
│   "Approve" / "Deny"                  │  │
└────────────┬──────────────────────────┘  │
             │                             │
             │ User taps "Approve"         │
             │                             │
             │ Callback webhook            │
             │                             │
┌────────────▼──────────────────────────┐  │
│   Forward request to real SSH agent   │◄─┘
│   (via SSH_AUTH_SOCK env var)         │
│                                        │
│   Receive signature response           │
└────────────┬──────────────────────────┘
             │
             │ Return signature
             │
┌────────────▼──────────────────────────┐
│   Git (continues push)                 │
│   ✓ Commit signed                      │
│   ✓ Changes pushed                     │
└────────────────────────────────────────┘
```

### Bio-Git Architecture

```
┌──────────────────────────────────────────────┐
│         Your Terminal                        │
│  $ git push                                  │
│  (needs HTTPS credential)                    │
└────────────┬─────────────────────────────────┘
             │
             │ git-credential get
             │ protocol=https
             │ host=github.com
             │
┌────────────▼─────────────────────────────────┐
│     Bio-Git Credential Helper                │
│  (stdin parsing)                             │
│                                              │
│  1. Parse credential request                │
│  2. Extract protocol & host                 │
└────────────┬─────────────────────────────────┘
             │
             │ Check OS Keyring
             │ (Service: bio-git)
             │ (Key: https://github.com)
             │
┌────────────▼─────────────────────────────────┐
│   Request Biometric Verification             │
│                                              │
│   Windows:  Windows.Security.Credentials    │
│             UserConsentVerifier             │
│                                              │
│   macOS:    LocalAuthentication Framework   │
│             LAContext.evaluatePolicy()      │
│                                              │
│   Linux:    polkit (DBus)                   │
│             org.freedesktop.PolicyKit       │
└────────────┬─────────────────────────────────┘
             │
      ┌──────┴──────┐
      │             │
  ✓ Verified   ✗ Denied
      │             │
      │      ┌──────▼─────────┐
      │      │ Return error   │
      │      │ (Git fails)    │
      │      └────────────────┘
      │
┌─────▼───────────────────────────┐
│ Retrieve from OS Keyring        │
│ Return to Git via stdout:       │
│ username=user                   │
│ password=token_from_keyring     │
└─────┬───────────────────────────┘
      │
┌─────▼───────────────────────────┐
│ Git Uses Credential             │
│ $ git push origin main          │
│ (authenticated, push succeeds)  │
└─────────────────────────────────┘
```

---

## Security Considerations

### Git-Sentry

**Threat Model:**
- ✅ Protects against: Malware attempting to sign commits
- ✅ Protects against: Automated scripts using your SSH key
- ⚠️ Does NOT protect against: Physical access to machine (after daemon start)
- ⚠️ Does NOT protect against: SSH key compromise via other means

**Best Practices:**
1. Keep Telegram bot token secure (use environment variables or secrets manager)
2. Monitor approval requests - sudden spike may indicate compromise
3. Set reasonable timeout (300s default)
4. Regularly review SSH key access logs
5. Consider revoking keys if unusual activity detected

### Bio-Git

**Threat Model:**
- ✅ Protects against: Background credential theft
- ✅ Protects against: Process injection attempts
- ✅ Protects against: Keylogger-based credential capture (requires biometric)
- ⚠️ Does NOT protect against: Physical access (user can just approve)
- ⚠️ Does NOT protect against: OS-level exploitation

**Best Practices:**
1. Enable biometric on your device (Windows Hello, Touch ID)
2. Use strong PAC credentials / PATs instead of passwords
3. Regularly rotate credentials in OS keyring
4. Monitor credential helper logs
5. Combine with other security measures (firewall, 2FA on GitHub)

---

## Troubleshooting

### Git-Sentry

**Problem:** "SSH_AUTH_SOCK not set"
```bash
# Solution: Set it explicitly
export SSH_AUTH_SOCK="/tmp/git-sentry.sock"
```

**Problem:** "Telegram API error: 401 Unauthorized"
```bash
# Solution: Check your bot token
git-sentry test --bot-token "your_token" --chat-id 12345
```

**Problem:** Daemon not receiving requests
```bash
# Check if socket exists
ls -la /tmp/git-sentry.sock

# Verify SSH_AUTH_SOCK points to it
echo $SSH_AUTH_SOCK

# Try a test SSH connection
ssh -v user@host
```

### Bio-Git

**Problem:** "No biometric device found"
```bash
# Windows: Enable Windows Hello
# Settings > Sign-in options > Biometric recognition

# macOS: Check Touch ID
# System Preferences > Touch ID & Password

# Linux: Install polkit
# sudo apt install policykit-1 libpolkit-agent-1
```

**Problem:** "Credential not found in keyring"
```bash
# First time access triggers storage:
git clone https://github.com/user/repo.git

# For existing repos, credentials must be stored first
git config --global credential.store bio
```

---

## Future Enhancements

### Git-Sentry
- [ ] Windows Named Pipes support
- [ ] Hardware security key integration (YubiKey)
- [ ] Request analytics dashboard
- [ ] Per-command approval policies
- [ ] Integration with Slack/Discord
- [ ] SQLite audit log

### Bio-Git
- [ ] Linux polkit integration
- [ ] PIN-based fallback authentication
- [ ] Credential expiration
- [ ] Per-repository PIN override
- [ ] Integration with hardware security keys
- [ ] Biometric cache (with timeout)

---

## Development

### Building

```bash
# Build both projects
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy

# Format code
cargo fmt
```

### Dependencies

**Git-Sentry:**
- `tokio` - Async runtime
- `bytes` - Binary protocol parsing
- `reqwest` - HTTP client for Telegram API
- `serde_json` - JSON handling
- `uuid` - Request ID generation
- `unix-socket` (Unix only) - Socket communication

**Bio-Git:**
- `keyring` - Cross-platform credential storage
- `windows` (Windows) - Windows Hello API
- `objc` (macOS) - Objective-C interop
- `serde_json` - JSON handling

---

## License

MIT or Apache-2.0 (choose either)

---

## Contributing

Pull requests welcome! Areas for contribution:
- Windows support for Git-Sentry
- Linux polkit integration for Bio-Git
- GUI dashboard for request approvals
- Additional notification backends (Slack, Discord, etc.)
- Security audit and feedback

---

## Author

Created as innovative security solutions for Git workflows.

---

## Support & Questions

- Check troubleshooting section above
- Review SSH Agent Protocol: https://tools.ietf.org/html/draft-miller-ssh-agent
- Git Credential Helper Docs: https://git-scm.com/docs/git-credential
- Telegram Bot API: https://core.telegram.org/bots/api
