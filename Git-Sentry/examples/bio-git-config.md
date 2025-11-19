# Bio-Git Configuration Examples

## Git Configuration

### Global Setup

```bash
# Set Bio-Git as global credential helper
git config --global credential.helper bio

# Verify
git config --global --list | grep credential.helper
# Output: credential.helper=bio
```

### Per-Repository Setup

```bash
cd /path/to/repo
git config credential.helper bio

# Verify
git config --local --list | grep credential.helper
```

### Chained Helpers (Fallback)

```bash
# Use Bio-Git first, then fall back to cached credentials
git config --global credential.helper "bio"
git config --global --add credential.helper "cache --timeout=3600"

# Or with store helper
git config --global credential.helper "bio"
git config --global --add credential.helper store
```

## Platform-Specific Setup

### macOS

Create ~/.config/git/bio-git.conf:

```ini
[biometric]
    provider = local-auth
    timeout = 30
    allow-fallback = false

[keychain]
    service = bio-git
    account-prefix = github_
```

### Windows

Create %APPDATA%\Git\bio-git.conf:

```ini
[biometric]
    provider = hello
    timeout = 30
    allow-fallback = false

[credential-manager]
    target-prefix = git:
```

### Linux

Create ~/.config/git/bio-git.conf:

```ini
[biometric]
    provider = polkit
    agent = gnome-keyring
    timeout = 30

[secret-service]
    collection = default
    unlock = true
```

## Environment Variables

```bash
# Log level
export RUST_LOG=debug,bio_git=trace

# Biometric timeout (seconds)
export BIO_GIT_TIMEOUT=30

# Force specific platform provider
export BIO_GIT_PROVIDER=hello  # Windows
export BIO_GIT_PROVIDER=local_auth  # macOS
export BIO_GIT_PROVIDER=polkit  # Linux
```

## Repository-Specific Credential Storage

### Store GitHub Token

```bash
# Interactive storage
echo "protocol=https
host=github.com
username=YOUR_USERNAME
password=ghp_YOUR_TOKEN" | git-credential-bio approve

# Verify storage
echo "protocol=https
host=github.com" | git-credential-bio get
```

### Store GitLab Token

```bash
echo "protocol=https
host=gitlab.com
username=oauth2
password=glpat-YOUR_TOKEN" | git-credential-bio approve
```

### Store Multiple Tokens

```bash
# Script to store multiple credentials
#!/bin/bash

REPOS=(
    "github.com:YOUR_USER:ghp_xxxxx"
    "gitlab.com:oauth2:glpat_xxxxx"
    "bitbucket.org:YOUR_USER:app_password_xxxxx"
)

for repo in "${REPOS[@]}"; do
    IFS=':' read -r host user pass <<< "$repo"
    echo "protocol=https
host=$host
username=$user
password=$pass" | git-credential-bio approve
done

echo "✓ All credentials stored"
```

## Shell Integration

Add to ~/.bashrc or ~/.zshrc:

### Function: Force Biometric

```bash
# Force biometric verification even if cached
git_secure_push() {
    # Clear temporary cache
    unset GIT_CREDENTIAL_CACHE
    
    # Require biometric
    git push "$@"
}

# Usage: git_secure_push origin main
```

### Function: List Stored Credentials

```bash
git_list_creds() {
    case "$(uname)" in
        Darwin)
            # macOS: List keychain entries
            security dump-keychain -d login.keychain | grep -A 2 "bio-git" | grep "acct"
            ;;
        MINGW*)
            # Windows: List Credential Manager
            cmdkey /list | grep "git:"
            ;;
        Linux)
            # Linux: List secret-service
            secret-tool search service bio-git
            ;;
    esac
}
```

### Function: Delete Credentials

```bash
git_delete_cred() {
    local host="${1:?Usage: git_delete_cred <host>}"
    
    echo "protocol=https
host=$host" | git-credential-bio reject
    
    echo "✓ Credential for $host deleted"
}
```

## GitHub Setup

### Create Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Click "Generate new token (classic)"
3. Select scopes:
   - ✓ repo (Full control of private repositories)
   - ✓ workflow (Update GitHub Action workflows)
   - ✓ gist (Create gists)
4. Set expiration (90 days recommended)
5. Click "Generate token"
6. Copy the token immediately

### Store Token with Bio-Git

```bash
# Store the token
read -p "Enter your GitHub token: " -s GITHUB_TOKEN
echo

echo "protocol=https
host=github.com
username=YOUR_USERNAME
password=$GITHUB_TOKEN" | git-credential-bio approve

# Verify it works
git clone https://github.com/YOUR_USERNAME/private-repo.git
```

## GitLab Setup

### Create Personal Access

1. Go to GitLab Profile → Access Tokens
2. Create new token with scopes:
   - ✓ read_api
   - ✓ read_repository
   - ✓ write_repository
3. Copy the token
4. Set expiration (90 days recommended)

### Store Token with Bio-Git

```bash
read -p "Enter your GitLab token: " -s GITLAB_TOKEN
echo

echo "protocol=https
host=gitlab.com
username=oauth2
password=$GITLAB_TOKEN" | git-credential-bio approve
```

## Testing Bio-Git

### Test Script

```bash
#!/bin/bash

echo "🔐 Testing Bio-Git Setup"
echo ""

# Check installation
if ! command -v git-credential-bio &> /dev/null; then
    echo "✗ git-credential-bio not found"
    exit 1
fi
echo "✓ git-credential-bio installed"

# Check Git config
HELPER=$(git config --global credential.helper)
if [ "$HELPER" != "bio" ]; then
    echo "✗ credential.helper is not set to 'bio' (current: $HELPER)"
    exit 1
fi
echo "✓ Git configured to use bio-git"

# Test credential retrieval
echo ""
echo "Testing credential retrieval..."
RESULT=$(echo "protocol=https
host=github.com" | git-credential-bio get 2>&1)

if [ -n "$RESULT" ]; then
    echo "✓ Credentials found"
else
    echo "⚠ No stored credentials (expected on first run)"
fi

echo ""
echo "✓ Bio-Git setup verified"
```

## Debugging

### Enable Debug Logging

```bash
# Set Rust log level
export RUST_LOG=debug,bio_git=trace

# Run credential helper in debug mode
echo "protocol=https
host=github.com" | RUST_LOG=debug git-credential-bio get
```

### Check Keyring Contents

#### macOS
```bash
# List all bio-git entries
security dump-keychain -d login.keychain | grep -A 3 "bio-git"

# Get specific entry
security find-internet-password -s bio-git -a github.com login.keychain
```

#### Windows
```bash
# List credentials
cmdkey /list | grep "git:"

# Use Credential Manager GUI
control /name Microsoft.CredentialManager
```

#### Linux
```bash
# List secrets
secret-tool search service bio-git

# View specific entry
secret-tool lookup service bio-git host github.com
```

### Manual Credential Testing

```bash
# Store a test credential
echo "protocol=https
host=test.example.com
username=testuser
password=testpass" | git-credential-bio approve

# Retrieve it
echo "protocol=https
host=test.example.com" | git-credential-bio get

# Delete it
echo "protocol=https
host=test.example.com" | git-credential-bio reject
```

## Production Checklist

- [ ] Biometric enrollment complete
- [ ] Git configured: `git config --global credential.helper bio`
- [ ] Test credentials stored
- [ ] Biometric timeout set appropriately
- [ ] Keyring backend verified
- [ ] Credentials in secure storage (not plaintext)
- [ ] Shell history doesn't contain tokens
- [ ] Regular token rotation schedule (90 days)
- [ ] Backup authentication method configured
- [ ] Monitoring/logging enabled

## Security Best Practices

1. **Use PATs instead of passwords:**
   ```bash
   # Good: Personal Access Token
   ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxx
   
   # Bad: GitHub password
   MyPassword123!
   ```

2. **Set reasonable timeout:**
   ```bash
   # Quick timeout for high-security environments
   export BIO_GIT_TIMEOUT=10  # 10 seconds
   
   # Longer for usability
   export BIO_GIT_TIMEOUT=60  # 1 minute
   ```

3. **Rotate credentials regularly:**
   ```bash
   # GitHub: Regenerate token every 90 days
   # GitLab: Rotate PAT quarterly
   ```

4. **Monitor access:**
   ```bash
   # Enable debug logging in scripts
   RUST_LOG=info
   ```

5. **Use fallback authentication:**
   ```bash
   # Configure SSH as primary
   # Use Bio-Git for HTTPS backup
   ```
