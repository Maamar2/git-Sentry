# Bio-Git: Git Credential Helper with Biometric Authentication

A custom Git credential helper that requires biometric verification (Touch ID, Windows Hello, Face ID) for every credential access.

## Quick Start

### 1. Build & Install

```bash
cargo build --release
cp target/release/git-credential-bio /usr/local/bin/
```

### 2. Configure Git

```bash
# Set Bio-Git as your credential helper
git config --global credential.helper bio

# Verify it's set
git config --global credential.helper
# Output: bio
```

### 3. Use It

```bash
# On first access, store credentials
git clone https://github.com/user/repo.git
# Biometric prompt appears
# Credentials are stored securely

# On subsequent access, biometric required again
git push origin main
# Biometric prompt appears again
# Operation proceeds after verification
```

## How It Works

```
Git needs credentials
        ↓
Git calls: git-credential-bio get
        ↓
Bio-Git checks OS keyring
        ↓
Biometric verification required
        ↓
User provides fingerprint/Face ID
        ↓
Credential retrieved from keyring
        ↓
Git receives token/password
        ↓
Git operation proceeds
```

## Supported Platforms

### macOS (Touch ID / Face ID)
- Requires: macOS 10.12+
- Uses: LocalAuthentication framework

### Windows (Windows Hello / Fingerprint)
- Requires: Windows 10/11
- Uses: Windows.Security.Credentials.UserConsentVerifier
- Setup: Settings > Sign-in options > Biometric recognition

### Linux (Polkit)
- Requires: libpolkit-agent-1, libsecret
- Status: ⚠️ Planned for v0.2
- Uses: org.freedesktop.PolicyKit

## Installation

### macOS

```bash
# Build for macOS
cargo build --release --target x86_64-apple-darwin

# Install
cp target/x86_64-apple-darwin/release/git-credential-bio \
   /usr/local/bin/

# Configure
git config --global credential.helper bio
```

### Windows

```bash
# Build for Windows
cargo build --release

# Copy to PATH (e.g., C:\Program Files\Git\mingw64\libexec\git-core\)
copy target\release\git-credential-bio.exe `
  "C:\Program Files\Git\mingw64\libexec\git-core\"

# Configure
git config --global credential.helper bio
```

### Linux

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install libsecret-1-dev libpolkit-agent-1

# Build
cargo build --release

# Install
sudo cp target/release/git-credential-bio /usr/local/bin/

# Configure
git config --global credential.helper bio
```

## Credential Storage

### Storage Format

Credentials are stored in the OS native credential manager:

**macOS Keychain:**
```
Service: bio-git
Account: <username>
Password: <token/password>
```

**Windows Credential Manager:**
```
Type: Generic
Target: bio-git
Username: <username>
Password: <token/password>
```

**Linux Secret Service:**
```
Collection: default
Label: bio-git: https://github.com
Attributes: protocol=https, host=github.com
```

### Key Format

```
<protocol>://<host>
```

Examples:
- `https://github.com` - GitHub HTTPS credentials
- `https://gitlab.com` - GitLab HTTPS credentials
- `ssh://git@github.com` - SSH key phrase (if stored)

## Configuration

### Per-Repository Setup

```bash
# Use Bio-Git for this repo only
cd my-repo
git config credential.helper bio

# Verify
git config credential.helper
# Output: bio
```

### Global Setup

```bash
# Use Bio-Git for all repos
git config --global credential.helper bio

# Verify
git config --global credential.helper
# Output: bio
```

### Mixed Setup

You can chain helpers:

```bash
# Use Bio-Git, then fall back to stored passwords
git config --global credential.helper 'bio'
git config --global credential.helper 'store'  # Falls back if bio denies
```

## Usage Examples

### First-Time Repository Access

```bash
$ git clone https://github.com/user/private-repo.git
Cloning into 'private-repo'...

┌─────────────────────────────────────┐
│ Bio-Git requires your Touch ID      │
│ to access credentials               │
│                                     │
│ [====] Scanning fingerprint...      │
└─────────────────────────────────────┘

remote: Enumerating objects...
Unpacking objects: 100%...
```

### Subsequent Operations

```bash
$ git push origin main

┌─────────────────────────────────────┐
│ Bio-Git requires your biometric     │
│ authentication to continue          │
│                                     │
│ [Face scan in progress...]          │
└─────────────────────────────────────┘

Enumerating objects: 42
Counting objects: 100% (42/42)
```

### Manual Credential Storage

```bash
# Store credentials without cloning
git config --global user.github-token "ghp_xxxxxxxxxxxxx"

# Or use the credential protocol directly
echo "protocol=https
host=github.com
username=user
password=token" | git-credential-bio approve

# Test retrieval
echo "protocol=https
host=github.com" | git-credential-bio get
```

## Troubleshooting

### Biometric Device Not Found

**macOS:**
```bash
# Check if Touch ID is available
system_profiler SPBiometricInfo

# Verify credential helper
git config --global credential.helper
```

**Windows:**
```bash
# Check Windows Hello status
Settings > Sign-in options > Biometric recognition

# Verify credential helper
git config --global credential.helper
```

### "User canceled biometric verification"

This is expected when:
- User denies the biometric prompt
- No fingerprint detected within timeout
- Biometric device not ready

Solution: Try the operation again and approve the biometric prompt.

### Credential Not Found in Keyring

```bash
# List stored credentials
git credential-bio get << EOF
protocol=https
host=github.com
EOF

# Store new credentials
git credential approve << EOF
protocol=https
host=github.com
username=user
password=token
EOF

# Verify storage
git credential-bio get << EOF
protocol=https
host=github.com
EOF
```

### "No biometric configured"

**macOS:**
```bash
# Enroll your fingerprint
System Preferences > Touch ID & Password > Add Fingerprint
```

**Windows:**
```bash
# Set up Windows Hello
Settings > Accounts > Sign-in options > Windows Hello Face
# Or: Windows Hello Fingerprint (if device supports)
```

## Security Considerations

### Threat Model

**Protects against:**
- ✅ Credential theft via malware background process
- ✅ Keylogger-based credential capture
- ✅ Malicious scripts attempting Git operations
- ✅ Unattended machine access

**Does NOT protect against:**
- ⚠️ Physical access (user can approve biometric)
- ⚠️ Keyring database corruption
- ⚠️ OS-level exploits (root access)
- ⚠️ Pre-stored credentials on disk

### Best Practices

1. **Use Personal Access Tokens (PATs) instead of passwords:**
   ```bash
   # Create token on GitHub: Settings > Developer settings > Personal access tokens
   # Use token as password
   git clone https://user:token@github.com/user/repo.git
   ```

2. **Set credential expiration:**
   - GitHub: Set PAT expiration to 90 days
   - Use different tokens for different machines

3. **Monitor biometric usage:**
   ```bash
   # Check keyring for unauthorized accounts
   security dump-keychain -d login.keychain  # macOS
   
   # Monitor credential manager
   # Windows: Credential Manager > Windows Credentials
   ```

4. **Disable credential caching fallback:**
   ```bash
   # Remove store helper to prevent fallback
   git config --global --unset-all credential.helper
   git config --global credential.helper bio
   ```

5. **Use with other security layers:**
   - 2FA on GitHub/GitLab
   - SSH key as primary method
   - This as secondary for HTTPS operations

## Advanced Configuration

### Custom Credential Protocols

Bio-Git supports any Git credential protocol:

```bash
# SSH key passphrase
echo "protocol=ssh
host=github.com
username=git" | git-credential-bio get

# GitLab token
echo "protocol=https
host=gitlab.com
username=oauth2" | git-credential-bio get
```

### Integration with Git Config

```bash
# Per-host configuration
git config --add credential.github.com.useHttpPath true
git config --add credential.github.com.helper bio

# Per-repository
cd my-repo
git config --local credential.helper bio
```

### Debugging

```bash
# Enable debug tracing
RUST_LOG=debug git-credential-bio get << EOF
protocol=https
host=github.com
EOF

# View keyring contents
# macOS: Keychain Access application
# Windows: Credential Manager (Control Panel)
# Linux: seahorse or secret-tool
```

## Development

```bash
# Build
cargo build --release

# Test
cargo test

# Check code quality
cargo clippy

# Format
cargo fmt

# Build for specific target
cargo build --release --target x86_64-apple-darwin
```

## Platform-Specific Notes

### macOS
- Uses CoreFoundation and LocalAuthentication frameworks
- Compatible with both Touch ID and Face ID
- Requires macOS 10.12+

### Windows
- Uses Windows Runtime (WinRT) APIs
- Requires Windows 10 version 2004+
- Works with Windows Hello Face and Fingerprint

### Linux
- Currently uses placeholder (polkit planned)
- Requires libsecret for credential storage
- Planned: Integration with polkit PAM module

## Related Projects

- **Git-Sentry**: SSH agent proxy with Telegram 2FA
- See parent directory `README.md` for comparison

## License

MIT or Apache-2.0

## Contributing

Contributions welcome! Especially:
- Linux polkit integration
- Additional platform support
- Performance improvements
- Security audit feedback

---

## References

- Git Credential Protocol: https://git-scm.com/docs/git-credential
- macOS LocalAuthentication: https://developer.apple.com/documentation/localauthentication
- Windows Hello API: https://docs.microsoft.com/en-us/windows/security/identity-protection/hello-for-business/
- Secret Service API: https://wiki.gnome.org/Projects/GnomeKeyring/SecretServiceAPI
