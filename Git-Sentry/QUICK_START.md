# Git-Sentry & Bio-Git: Quick Reference Guide

## 🚀 Fast Start (5 minutes)

### Git-Sentry Setup

```bash
# 1. Get Telegram Bot Token
# → Message @BotFather on Telegram
# → Create bot, copy token

# 2. Get your Chat ID
curl https://api.telegram.org/bot<TOKEN>/getUpdates | grep '"id"'

# 3. Build
cd git-sentry
cargo build --release

# 4. Test
./target/release/git-sentry test \
  --bot-token "YOUR_TOKEN" \
  --chat-id YOUR_CHAT_ID

# 5. Run
export GIT_SENTRY_BOT_TOKEN="YOUR_TOKEN"
export GIT_SENTRY_CHAT_ID=YOUR_CHAT_ID
export SSH_AUTH_SOCK="/tmp/git-sentry.sock"

./target/release/git-sentry daemon
```

### Bio-Git Setup

```bash
# 1. Build
cd bio-git
cargo build --release

# 2. Install
cp target/release/git-credential-bio /usr/local/bin/

# 3. Configure
git config --global credential.helper bio

# 4. Use
git clone https://github.com/user/repo.git
# (Will ask for biometric)
```

---

## 📋 Command Reference

### Git-Sentry

```bash
# Start daemon with all options
git-sentry daemon \
  --bot-token "your_token" \
  --chat-id 1234567890 \
  --socket /tmp/git-sentry.sock \
  --ssh-auth-sock /run/user/1000/ssh.socket \
  --timeout 300

# Test Telegram connection
git-sentry test --bot-token "token" --chat-id 123

# Show setup instructions
git-sentry setup --bot-token "token" --chat-id 123

# Environment variables
export GIT_SENTRY_BOT_TOKEN="token"
export GIT_SENTRY_CHAT_ID=123
export GIT_SENTRY_SOCKET="/tmp/git-sentry.sock"
export SSH_AUTH_SOCK="/tmp/git-sentry.sock"  # Point to Git-Sentry
```

### Bio-Git

```bash
# Configure globally
git config --global credential.helper bio

# Configure per-repo
git config credential.helper bio

# Store credentials
echo "protocol=https
host=github.com
username=user
password=token" | git-credential-bio approve

# Retrieve credentials
echo "protocol=https
host=github.com" | git-credential-bio get

# Delete credentials
echo "protocol=https
host=github.com" | git-credential-bio reject
```

---

## 🔧 Configuration Files

### Git-Sentry Systemd Service

**File:** `~/.config/systemd/user/git-sentry.service`

```ini
[Unit]
Description=Git-Sentry
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/git-sentry daemon \
  --bot-token YOUR_TOKEN \
  --chat-id YOUR_CHAT_ID
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
```

**Commands:**
```bash
systemctl --user enable git-sentry
systemctl --user start git-sentry
systemctl --user status git-sentry
journalctl --user -u git-sentry -f  # Watch logs
```

### Git Config

**Global:** `~/.gitconfig`
```ini
[credential]
    helper = bio
```

**Per-repo:** `.git/config`
```ini
[credential]
    helper = bio
```

---

## 🐛 Troubleshooting Cheat Sheet

### Git-Sentry

| Problem | Solution |
|---------|----------|
| Socket not found | Check `ls -la $SSH_AUTH_SOCK` |
| Telegram error | Run: `git-sentry test --bot-token "..." --chat-id ...` |
| No approve prompt | Verify real SSH agent: `echo $SSH_AUTH_SOCK` |
| Timeout | Increase with `--timeout 600` |
| Daemon won't start | Check logs: `journalctl --user -u git-sentry` |

### Bio-Git

| Problem | Solution |
|---------|----------|
| Biometric not working | Check system settings (Touch ID/Hello) |
| Credential not found | First: `git clone` to store credentials |
| Helper not configured | Run: `git config --global credential.helper bio` |
| Permission denied | Check: `ls -la $(which git-credential-bio)` |

---

## 🔐 Security Checklist

### For Git-Sentry
- [ ] Bot token stored in env var (not in shell history)
- [ ] SSH key loaded: `ssh-add -l`
- [ ] Socket created: `ls -la $SSH_AUTH_SOCK`
- [ ] Telegram test passed: `git-sentry test`
- [ ] First git command approved on phone

### For Bio-Git
- [ ] Biometric enrolled on device
- [ ] `credential.helper bio` configured
- [ ] First clone/push stored credentials
- [ ] Biometric prompt appears on each use
- [ ] No plaintext tokens in shell history

---

## 📊 Project Structure

```
Git-Sentry/
├── README.md                          # Main docs
├── SETUP_SUMMARY.md                   # Complete summary
├── .gitignore
│
├── git-sentry/                        # SSH proxy
│   ├── Cargo.toml                     # Dependencies
│   ├── README.md                      # Setup guide
│   └── src/
│       ├── main.rs                    # CLI entry
│       ├── daemon.rs                  # Socket listener
│       ├── ssh_protocol.rs            # Protocol parser
│       ├── telegram.rs                # Telegram API
│       └── proxy.rs                   # Forwarding logic
│
├── bio-git/                           # Credential helper
│   ├── Cargo.toml                     # Dependencies
│   ├── README.md                      # Setup guide
│   └── src/
│       ├── main.rs                    # Entry point
│       ├── credential_helper.rs       # Protocol handler
│       ├── biometric.rs               # OS APIs
│       └── keyring_mgr.rs             # Secure storage
│
└── examples/
    ├── git-sentry-config.md           # Config examples
    └── bio-git-config.md              # Config examples
```

---

## 🎯 Common Workflows

### Add Git-Sentry to Existing Setup

```bash
# 1. Make sure SSH agent is running
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_rsa

# 2. Save original socket path
export REAL_SSH_AUTH_SOCK=$SSH_AUTH_SOCK

# 3. Start Git-Sentry pointing to real agent
git-sentry daemon \
  --bot-token "your_token" \
  --chat-id 123456 \
  --ssh-auth-sock $REAL_SSH_AUTH_SOCK

# 4. In new terminal, point to Git-Sentry
export SSH_AUTH_SOCK="/tmp/git-sentry.sock"

# 5. Test
git clone git@github.com:user/repo.git
# (Will require Telegram approval)
```

### Use Both Bio-Git and SSH Key

```bash
# Git-Sentry handles SSH (for commits/pushes)
export SSH_AUTH_SOCK="/tmp/git-sentry.sock"

# Bio-Git handles HTTPS (for package managers)
git config --global credential.helper bio

# Clone over SSH (uses Git-Sentry)
git clone git@github.com:user/repo.git

# Or clone over HTTPS (uses Bio-Git biometric)
git clone https://github.com/user/repo.git
```

### Multiple Machines

```bash
# On machine 1: Run Git-Sentry daemon
ssh machine1.local
git-sentry daemon --bot-token "token" --chat-id 123

# On machine 2: Use Bio-Git
git config --global credential.helper bio
git clone https://github.com/user/repo.git
```

---

## 📚 Documentation Map

| Document | Contains |
|----------|----------|
| `README.md` | Overview, architecture, both projects |
| `git-sentry/README.md` | Git-Sentry setup, protocol details |
| `bio-git/README.md` | Bio-Git setup, biometric APIs |
| `examples/git-sentry-config.md` | Config templates, systemd service |
| `examples/bio-git-config.md` | Git configs, credential storage |
| `SETUP_SUMMARY.md` | This setup guide with inventory |

---

## 🚀 Next Steps

1. **Read** the main `README.md` (10 min overview)
2. **Pick** one project to start (Git-Sentry is simpler)
3. **Build** with `cargo build --release` (2-3 min)
4. **Configure** using `examples/` templates (5 min)
5. **Test** with provided test commands (2 min)
6. **Use** in your daily Git workflow

---

## ❓ FAQ

**Q: Can I use both projects together?**
A: Yes! Use Git-Sentry for SSH operations (commits), Bio-Git for HTTPS (package managers).

**Q: Is this secure?**
A: Yes, but it's a first layer of defense. Combine with OS security, 2FA on GitHub/GitLab, VPN, etc.

**Q: What if Telegram is down?**
A: Requests timeout and Git operations fail. You can adjust timeout or use fallback auth.

**Q: Can I use this on Windows?**
A: Git-Sentry: Windows via WSL2. Bio-Git: Full Windows support via Windows Hello.

**Q: How do I rotate credentials?**
A: For Bio-Git, delete from keyring and store new ones. For Git-Sentry, SSH key management is separate.

**Q: What about CI/CD pipelines?**
A: Disable Git-Sentry for automated keys, use separate deployment keys without Git-Sentry.

---

## 📞 Support

See troubleshooting sections in:
- `git-sentry/README.md` (Git-Sentry issues)
- `bio-git/README.md` (Bio-Git issues)
- Main `README.md` (Architecture questions)

---

**Last Updated:** 2025-11-19
**Version:** 0.1.0
