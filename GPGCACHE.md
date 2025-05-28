# Managing GPG Passphrase Caching

GPG (GNU Privacy Guard) caches your passphrase for a certain period by default. This caching behavior can be configured to suit your security needs. This document explains how to manage GPG passphrase caching.

---

## Default Behavior

By default, GPG caches your passphrase for 10 minutes. This means you won't need to re-enter your passphrase for subsequent operations within this time frame.

---

## Configuring GPG Caching

You can configure the caching behavior by modifying the `gpg-agent` configuration file.

### Steps to Configure:

1. **Locate the Configuration File**:
   The configuration file is typically located at:
   ```
   ~/.gnupg/gpg-agent.conf
   ```

   If the file does not exist, create it:
   ```bash
   touch ~/.gnupg/gpg-agent.conf
   ```

2. **Set Cache Durations**:
   Add or modify the following lines in the file:
   ```
   default-cache-ttl 600
   max-cache-ttl 7200
   ```

   - `default-cache-ttl`: The time (in seconds) a passphrase is cached after the last use. In this example, it is set to 10 minutes (600 seconds).
   - `max-cache-ttl`: The maximum time (in seconds) a passphrase is cached, even if it is used multiple times. In this example, it is set to 2 hours (7200 seconds).

3. **Apply the Changes**:
   Restart the GPG agent to apply the changes:
   ```bash
   gpgconf --kill gpg-agent
   ```

   The agent will automatically restart the next time it is needed.

---

## Disabling Caching

If you want to disable passphrase caching entirely, set the cache durations to `0`:
```
default-cache-ttl 0
max-cache-ttl 0
```

This will require you to enter your passphrase every time a password is decrypted.

---

## Checking the Current Configuration

To check the current caching settings, run:
```bash
gpgconf --list-options gpg-agent
```

---

## Security Considerations

- **Shorter Cache Durations**: Use shorter cache durations for higher security, especially on shared or less secure systems.
- **Disabling Caching**: Disabling caching entirely is the most secure option but may be inconvenient for frequent use, as you will need to enter your password on every password decrypt.
- **Secure Your Environment**: Ensure your system is secure to prevent unauthorized access to cached passphrases.

---

For more details, refer to the [GPG Agent Manual](https://www.gnupg.org/documentation/manuals/gnupg/Agent-Options.html).
