use anyhow::Result;
use tracing::info;

/// Verify user presence through biometric authentication
/// 
/// This function attempts to verify the user through OS-specific biometric APIs:
/// - Windows: Windows Hello (Windows.Security.Credentials.UserConsentVerifier)
/// - macOS: LocalAuthentication framework (Touch ID / Face ID)
/// - Linux: polkit user authentication
pub async fn verify_user_presence() -> Result<()> {
    #[cfg(windows)]
    {
        verify_windows_hello().await
    }

    #[cfg(target_os = "macos")]
    {
        verify_macos_touch_id().await
    }

    #[cfg(target_os = "linux")]
    {
        verify_linux_polkit().await
    }

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("Biometric verification not supported on this platform")
    }
}

#[cfg(windows)]
async fn verify_windows_hello() -> Result<()> {
    use windows::Security::Credentials::UserConsentVerifier;
    use windows::Security::Credentials::UserConsentVerificationResult;

    info!("Requesting Windows Hello verification");

    // Check if biometric is available
    let available = UserConsentVerifier::CheckAvailabilityAsync()
        .await?
        .GetResults();

    match available {
        UserConsentVerificationResult::Available => {
            // Request user verification
            let result = UserConsentVerifier::RequestVerificationAsync(
                "Bio-Git requires your biometric confirmation to access credentials"
            )
            .await?
            .GetResults();

            match result {
                UserConsentVerificationResult::Verified => {
                    info!("✓ Biometric verification successful");
                    Ok(())
                }
                UserConsentVerificationResult::DeviceNotPresent => {
                    anyhow::bail!("No biometric device found")
                }
                UserConsentVerificationResult::NotConfigured => {
                    anyhow::bail!("Biometric not configured on this device")
                }
                UserConsentVerificationResult::DisabledByPolicy => {
                    anyhow::bail!("Biometric disabled by system policy")
                }
                UserConsentVerificationResult::UserCanceled => {
                    anyhow::bail!("User canceled biometric verification")
                }
                _ => anyhow::bail!("Biometric verification failed"),
            }
        }
        UserConsentVerificationResult::DeviceNotPresent => {
            anyhow::bail!("No biometric device found")
        }
        UserConsentVerificationResult::NotConfigured => {
            anyhow::bail!("Biometric not configured on this device")
        }
        _ => anyhow::bail!("Biometric not available"),
    }
}

#[cfg(target_os = "macos")]
async fn verify_macos_touch_id() -> Result<()> {
    use objc::msg_send;
    use objc::sel;
    use objc::sel_impl;
    use objc::runtime::Object;
    use objc_foundation::{NSString, INSString};

    info!("Requesting macOS Touch ID verification");

    unsafe {
        let context: *mut Object = msg_send![class!(LAContext), new];
        let mut error: *mut Object = std::ptr::null_mut();

        let reason = NSString::from_str("Bio-Git requires your Touch ID to access credentials");
        let can_eval: bool = msg_send![context, canEvaluatePolicy:1 error:&mut error];

        if !can_eval {
            anyhow::bail!("Touch ID not available on this device");
        }

        let success: bool = msg_send![context, evaluatePolicy:1 
                                      localizedReason:reason 
                                      reply:^(_success: bool, _error: *mut Object) {}];

        if success {
            info!("✓ Touch ID verification successful");
            Ok(())
        } else {
            anyhow::bail!("Touch ID verification failed")
        }
    }
}

#[cfg(target_os = "linux")]
async fn verify_linux_polkit() -> Result<()> {
    info!("Requesting Linux polkit authentication");

    // Note: This would typically use D-Bus and polkit
    // For now, a placeholder implementation
    anyhow::bail!("Linux polkit implementation not yet available - use password-based helpers")
}
