# Git-Sentry Configuration Examples

## Environment Variables Setup

Save this as ~/.git-sentry-env and source it:

```bash
#!/bin/bash

# Telegram Bot Configuration
export GIT_SENTRY_BOT_TOKEN="your_bot_token_here"
export GIT_SENTRY_CHAT_ID=1234567890

# Socket Configuration
export GIT_SENTRY_SOCKET="/tmp/git-sentry.sock"

# Real SSH Agent Socket (adjust based on your setup)
# For macOS with ssh-agent
export SSH_AUTH_SOCK="$HOME/.ssh/agent.sock"

# For Linux with systemd socket
# export SSH_AUTH_SOCK="${XDG_RUNTIME_DIR}/ssh.socket"

# For Linux with standard ssh-agent
# export SSH_AUTH_SOCK="/tmp/ssh-XXXXXXX/agent.XXXXX"

# Approval timeout in seconds (5 minutes = 300)
export GIT_SENTRY_APPROVAL_TIMEOUT=300
```

Usage:
```bash
source ~/.git-sentry-env
git-sentry daemon
```

## Systemd User Service

Save as ~/.config/systemd/user/git-sentry.service:

```ini
[Unit]
Description=Git-Sentry SSH Agent Proxy
Documentation=https://github.com/your-org/git-sentry
After=network.target

[Service]
Type=notify
ExecStart=/usr/local/bin/git-sentry daemon \
  --bot-token %E/GIT_SENTRY_BOT_TOKEN \
  --chat-id %E/GIT_SENTRY_CHAT_ID \
  --ssh-auth-sock %E/REAL_SSH_AUTH_SOCK \
  --timeout 300
Restart=on-failure
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true

# Environment
Environment="GIT_SENTRY_BOT_TOKEN=your_token_here"
Environment="GIT_SENTRY_CHAT_ID=1234567890"
Environment="REAL_SSH_AUTH_SOCK=/run/user/1000/ssh.socket"

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=git-sentry

[Install]
WantedBy=default.target
```

Enable it:
```bash
mkdir -p ~/.config/systemd/user/
systemctl --user enable git-sentry
systemctl --user start git-sentry
systemctl --user status git-sentry
```

Monitor logs:
```bash
journalctl --user -u git-sentry -f
```

## Shell Profile Integration

Add to your ~/.bashrc or ~/.zshrc:

```bash
# Git-Sentry Configuration
if [ -f ~/.git-sentry-env ]; then
    source ~/.git-sentry-env
fi

# Function to start Git-Sentry daemon if not running
git_sentry_start() {
    if [ ! -S "$GIT_SENTRY_SOCKET" ]; then
        echo "Starting Git-Sentry daemon..."
        git-sentry daemon &
        sleep 1
        if [ -S "$GIT_SENTRY_SOCKET" ]; then
            echo "✓ Git-Sentry started"
        else
            echo "✗ Failed to start Git-Sentry"
            return 1
        fi
    fi
}

# Auto-start on shell init
git_sentry_start
```

## Docker Deployment

Create a Dockerfile:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/git-sentry /usr/local/bin/
ENTRYPOINT ["git-sentry", "daemon"]
```

Build and run:
```bash
docker build -t git-sentry .
docker run -e GIT_SENTRY_BOT_TOKEN=xxx \
           -e GIT_SENTRY_CHAT_ID=yyy \
           -e SSH_AUTH_SOCK=/run/ssh.socket \
           -v /run/user/1000/ssh.socket:/run/ssh.socket \
           git-sentry
```

## SSH Config Integration

Add to ~/.ssh/config for automatic Git-Sentry proxying:

```
Host github.com
    User git
    IdentityFile ~/.ssh/id_rsa
    # Git-Sentry will intercept this
```

For repositories requiring Git-Sentry approval:

```
Host github-secure
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_secure
    # This key requires Git-Sentry approval
```

## Telegram Bot Setup Script

Save as setup_telegram.sh:

```bash
#!/bin/bash

echo "🤖 Git-Sentry Telegram Bot Setup"
echo ""
read -p "Enter Telegram Bot Token: " BOT_TOKEN
read -p "Enter your Chat ID: " CHAT_ID

echo ""
echo "Testing connection..."

# Test API connection
RESPONSE=$(curl -s https://api.telegram.org/bot${BOT_TOKEN}/getMe)

if echo $RESPONSE | grep -q '"ok":true'; then
    echo "✓ Bot token is valid"
    
    # Send test message
    curl -s -X POST \
        https://api.telegram.org/bot${BOT_TOKEN}/sendMessage \
        -H "Content-Type: application/json" \
        -d "{\"chat_id\": $CHAT_ID, \"text\": \"🔐 Git-Sentry test message - connection successful!\"}"
    
    echo ""
    echo "✓ Test message sent"
    echo ""
    echo "Add these to your environment:"
    echo "export GIT_SENTRY_BOT_TOKEN=\"$BOT_TOKEN\""
    echo "export GIT_SENTRY_CHAT_ID=$CHAT_ID"
else
    echo "✗ Invalid bot token"
    exit 1
fi
```

Usage:
```bash
chmod +x setup_telegram.sh
./setup_telegram.sh
```

## Testing Git-Sentry

Create a test script:

```bash
#!/bin/bash

set -e

echo "Testing Git-Sentry installation..."

# Check installation
if ! command -v git-sentry &> /dev/null; then
    echo "✗ git-sentry not found in PATH"
    exit 1
fi
echo "✓ git-sentry found"

# Check environment
if [ -z "$GIT_SENTRY_BOT_TOKEN" ]; then
    echo "✗ GIT_SENTRY_BOT_TOKEN not set"
    exit 1
fi
echo "✓ GIT_SENTRY_BOT_TOKEN is set"

if [ -z "$GIT_SENTRY_CHAT_ID" ]; then
    echo "✗ GIT_SENTRY_CHAT_ID not set"
    exit 1
fi
echo "✓ GIT_SENTRY_CHAT_ID is set"

# Test Telegram connection
echo ""
echo "Testing Telegram connection..."
git-sentry test --bot-token "$GIT_SENTRY_BOT_TOKEN" --chat-id "$GIT_SENTRY_CHAT_ID"

echo ""
echo "✓ All tests passed!"
```

## Production Checklist

- [ ] Telegram bot token stored securely (not in shell history)
- [ ] SSH_AUTH_SOCK pointing to correct socket
- [ ] Git-Sentry daemon running (systemd or similar)
- [ ] Notification timeout set appropriately (300s default)
- [ ] Logs being monitored (journalctl)
- [ ] SSH key permissions correct (600)
- [ ] Firewall allows Telegram API access
- [ ] Regular approval monitoring for anomalies
