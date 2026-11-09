use anyhow::{Context, Result, bail};
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

pub const DEFAULT_CLIPBOARD_TIMEOUT: Duration = Duration::from_secs(30);

pub trait ClipboardProvider {
    fn set_text(&mut self, text: &str) -> Result<()>;
    fn get_text(&mut self) -> Result<String>;
    fn clear(&mut self) -> Result<()>;
}

/// Cross-platform system clipboard implementation.
pub struct SystemClipboardProvider {
    clipboard: arboard::Clipboard,
}

impl SystemClipboardProvider {
    pub fn new() -> Result<Self> {
        let clipboard =
            arboard::Clipboard::new().context("Failed to initialise the system clipboard")?;

        Ok(Self { clipboard })
    }
}

impl ClipboardProvider for SystemClipboardProvider {
    fn set_text(&mut self, text: &str) -> Result<()> {
        self.clipboard
            .set_text(text)
            .context("Failed to write to the system clipboard")
    }

    fn get_text(&mut self) -> Result<String> {
        self.clipboard
            .get_text()
            .context("Failed to read the system clipboard")
    }

    fn clear(&mut self) -> Result<()> {
        self.clipboard
            .clear()
            .context("Failed to clear the system clipboard")
    }
}

/// Tracks a secret copied to the clipboard and clears it after a timeout.
pub struct ClipboardManager<P: ClipboardProvider> {
    provider: P,
    clear_after: Duration,
    active_text: Option<Zeroizing<String>>,
    expires_at: Option<Instant>,
}

impl<P: ClipboardProvider> ClipboardManager<P> {
    pub fn new(provider: P) -> Result<Self> {
        Self::with_timeout(provider, DEFAULT_CLIPBOARD_TIMEOUT)
    }

    pub fn with_timeout(provider: P, clear_after: Duration) -> Result<Self> {
        if clear_after.is_zero() {
            bail!("Clipboard timeout must be greater than zero");
        }

        Ok(Self {
            provider,
            clear_after,
            active_text: None,
            expires_at: None,
        })
    }

    /// Copies text and returns the expiry .
    pub fn copy(&mut self, text: &str) -> Result<Instant> {
        self.copy_at(text, Instant::now())
    }

    fn copy_at(&mut self, text: &str, now: Instant) -> Result<Instant> {
        if text.is_empty() {
            bail!("Refusing to copy an empty secret");
        }

        let expires_at = now
            .checked_add(self.clear_after)
            .context("Clipboard timeout is too large")?;

        let text = Zeroizing::new(text.to_owned());
        self.provider
            .set_text(text.as_str())
            .context("Could not copy the secret")?;

        self.active_text = Some(text);
        self.expires_at = Some(expires_at);

        Ok(expires_at)
    }

    /// How long remains until the current clipboard copy expires
    pub fn time_until_clear(&self) -> Option<Duration> {
        self.expires_at
            .map(|deadline| deadline.saturating_duration_since(Instant::now()))
    }

    pub fn is_active(&self) -> bool {
        self.active_text.is_some()
    }

    /// Clears the clipboard when the timeout has elapsed and has
    pub fn clear_if_expired(&mut self) -> Result<bool> {
        self.clear_if_expired_at(Instant::now())
    }

    fn clear_if_expired_at(&mut self, now: Instant) -> Result<bool> {
        match self.expires_at {
            Some(deadline) if now >= deadline => self.clear_if_unchanged(),
            _ => Ok(false),
        }
    }

    /// Clears the secret
    pub fn clear_now(&mut self) -> Result<bool> {
        self.clear_if_unchanged()
    }

    // TODO not loving this fn chain

    fn clear_if_unchanged(&mut self) -> Result<bool> {
        let Some(expected) = self.active_text.as_ref() else {
            return Ok(false);
        };

        // Zeroize that shit
        let actual = Zeroizing::new(
            self.provider
                .get_text()
                .context("Could not check clipboard contents before clearing")?,
        );

        if actual.as_str() != expected.as_str() {
            self.rm();
            return Ok(false);
        }

        self.provider
            .clear()
            .context("Could not clear the copied secret")?;

        self.rm();
        Ok(true)
    }

    fn rm(&mut self) {
        // Taking the value drops and zeroizes its buffer.
        self.active_text.take();
        self.expires_at = None;
    }
}

impl ClipboardManager<SystemClipboardProvider> {
    /// Use os clipboard.
    pub fn system() -> Result<Self> {
        Self::new(SystemClipboardProvider::new()?)
    }

    pub fn system_with_timeout(timeout: Duration) -> Result<Self> {
        Self::with_timeout(SystemClipboardProvider::new()?, timeout)
    }
}

impl<P: ClipboardProvider> Drop for ClipboardManager<P> {
    fn drop(&mut self) {
        let _ = self.clear_if_unchanged();
    }
}

// Most tests LLM generated, I would LOVE to write them manually but im.... busy
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct MockClipboard {
        value: Arc<Mutex<Option<String>>>,
    }

    impl ClipboardProvider for MockClipboard {
        fn set_text(&mut self, text: &str) -> Result<()> {
            *self.value.lock().unwrap() = Some(text.to_owned());
            Ok(())
        }

        fn get_text(&mut self) -> Result<String> {
            self.value
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| anyhow!("Clipboard is empty"))
        }

        fn clear(&mut self) -> Result<()> {
            *self.value.lock().unwrap() = None;
            Ok(())
        }
    }

    #[test]
    fn test_clipboard_expires_and_clears() {
        let provider = MockClipboard::default();
        let shared = provider.value.clone();
        let mut manager =
            ClipboardManager::with_timeout(provider, Duration::from_secs(30)).unwrap();

        let now = Instant::now();
        let deadline = manager.copy_at("test-secret", now).unwrap();

        assert_eq!(*shared.lock().unwrap(), Some("test-secret".to_owned()));
        assert!(
            !manager
                .clear_if_expired_at(deadline - Duration::from_nanos(1))
                .unwrap()
        );

        assert!(manager.clear_if_expired_at(deadline).unwrap());
        assert_eq!(*shared.lock().unwrap(), None);
        assert!(!manager.is_active());
    }

    #[test]
    fn test_does_not_clear_replaced_clipboard_contents() {
        let provider = MockClipboard::default();
        let shared = provider.value.clone();
        let mut manager =
            ClipboardManager::with_timeout(provider, Duration::from_secs(30)).unwrap();

        let now = Instant::now();
        let deadline = manager.copy_at("fmp-secret", now).unwrap();

        *shared.lock().unwrap() = Some("something copied later".to_owned());

        assert!(!manager.clear_if_expired_at(deadline).unwrap());
        assert_eq!(
            *shared.lock().unwrap(),
            Some("something copied later".to_owned())
        );
        assert!(!manager.is_active());
    }

    #[test]
    fn test_manual_clear() {
        let provider = MockClipboard::default();
        let shared = provider.value.clone();
        let mut manager = ClipboardManager::new(provider).unwrap();

        manager.copy("manual-clear-secret").unwrap();

        assert!(manager.clear_now().unwrap());
        assert_eq!(*shared.lock().unwrap(), None);
    }

    #[test]
    fn test_drop_attempts_to_clear_secret() {
        let provider = MockClipboard::default();
        let shared = provider.value.clone();

        {
            let mut manager = ClipboardManager::new(provider).unwrap();
            manager.copy("drop-secret").unwrap();
        }

        assert_eq!(*shared.lock().unwrap(), None);
    }
}
