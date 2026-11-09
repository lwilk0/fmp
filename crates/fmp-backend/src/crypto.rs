use sequoia_openpgp::{
    KeyHandle,
    anyhow::{Context, Result, bail},
    armor::{Kind, Writer},
    cert::{Cert, CertBuilder, CipherSuite},
    crypto::KeyPair,
    crypto::{Password, SessionKey},
    packet::{PKESK, SKESK},
    parse::Parse,
    parse::stream::DecryptionHelper,
    parse::stream::{MessageLayer, MessageStructure, VerificationHelper},
    policy::StandardPolicy,
    serialize::Serialize,
    types::SymmetricAlgorithm,
};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

/// Handles decryption using one vault's private certificate.
pub struct VaultDecryptHelper {
    pub_cert: Cert,
    decryption_keypairs: Vec<KeyPair>,
}

impl VaultDecryptHelper {
    pub fn new(pub_cert: Cert, decryption_keypairs: Vec<KeyPair>) -> Self {
        Self {
            pub_cert,
            decryption_keypairs,
        }
    }
}

/// I hope one day I can understand why this works lol - info goes in one ear and out the other - I guess its something
impl DecryptionHelper for VaultDecryptHelper {
    fn decrypt(
        &mut self,
        pkesks: &[PKESK],
        _skesks: &[SKESK],
        sym_algo: Option<SymmetricAlgorithm>,
        decrypt: &mut dyn FnMut(Option<SymmetricAlgorithm>, &SessionKey) -> bool,
    ) -> Result<Option<Cert>> {
        for pkesk in pkesks {
            for unlocked_keypair in &self.decryption_keypairs {
                let mut keypair = unlocked_keypair.clone();

                let Some((algorithm, session_key)) = pkesk.decrypt(&mut keypair, sym_algo) else {
                    continue;
                };

                if decrypt(algorithm, &session_key) {
                    return Ok(Some(self.pub_cert.clone()));
                }
            }
        }

        bail!("No vault encryption key could decrypt the message")
    }
}

impl VerificationHelper for VaultDecryptHelper {
    fn get_certs(&mut self, _ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
        Ok(vec![self.pub_cert.clone()])
    }

    // OpenPGP rust code has got to be some of the worst looking stuff ever - its so ugly.
    // The checks arent redundant, its a security feature!
    fn check(&mut self, structure: MessageStructure) -> sequoia_openpgp::Result<()> {
        let mut layers = structure.into_iter();
        match layers.next() {
            Some(MessageLayer::Encryption { .. }) => {}
            _ => {
                sequoia_openpgp::anyhow::bail!("Account message is not encrypted");
            }
        }
        let results = match layers.next() {
            Some(MessageLayer::SignatureGroup { results }) => results,
            _ => {
                sequoia_openpgp::anyhow::bail!("Account message does not contain a signature");
            }
        };
        if layers.next().is_some() {
            sequoia_openpgp::anyhow::bail!("Unexpected OpenPGP message structure");
        }
        let good_signature = match results.as_slice() {
            [Ok(good)] => good,
            _ => {
                sequoia_openpgp::anyhow::bail!(
                    "Account signature is missing, invalid or ambiguous"
                );
            }
        };
        if good_signature.ka.cert().fingerprint() != self.pub_cert.fingerprint() {
            sequoia_openpgp::anyhow::bail!("Account was not signed by this vault");
        }

        Ok(())
    }
}

pub struct RecipientBuilder {
    recipient: String,
    vault_path: PathBuf,
}

impl RecipientBuilder {
    /// Construct a new `RecipientBuilder` with any parameters that implement the needed traits.
    pub fn new(recipient: impl Into<String>, vault_path: impl Into<PathBuf>) -> Self {
        Self {
            recipient: recipient.into(),
            vault_path: vault_path.into(),
        }
    }

    /// Creates the neccissary recipient files and adds them to the vault
    pub fn write(&self, password: &str) -> Result<()> {
        // Preemtive checks
        if self.recipient.trim().is_empty() {
            bail!("Recipient identity cannot be empty");
        }
        if password.is_empty() {
            bail!("Password cannot be empty");
        }
        create_owned_dir(&self.vault_path)?;

        let recipient_dir = self.vault_path.join("recipient");
        if recipient_dir.exists() {
            bail!("Recipient certificate already exists");
        }

        // Create everything in a temp directory before moving to real dir
        let temp = tempfile::Builder::new()
            .prefix(".recipient-")
            .tempdir_in(&self.vault_path)
            .context("Failed to create temp directory")?;

        let temp_dir = temp.path().join("recipient");

        create_owned_dir(&temp_dir)?;

        let password = Password::from(password);

        // Build cert
        let (cert, revocation) = CertBuilder::general_purpose(Some(self.recipient.as_str()))
            .set_cipher_suite(CipherSuite::Cv25519)
            .set_password(Some(password))
            .set_validity_period(None)
            .generate()
            .context("Failed to generate OpenPGP cert")?;

        // Write public cert(recipient.asc) - contains primary public key, user ID, associated cert info
        {
            let path = temp_dir.join("recipient.asc");
            let mut file = create_owned_file(&path)?;

            cert.armored()
                .serialize(&mut file)
                .context("Failed to serialize puplic certification")?;

            file.sync_all()?;
        }

        // Write private cert (recipient.key) - contains public cert info with private key.
        {
            let path = temp_dir.join("recipient.key");
            let mut file = create_owned_file(&path)?;

            let mut armor = Writer::new(&mut file, Kind::SecretKey)?;

            cert.as_tsk().serialize(&mut armor)?;

            armor.finalize()?;
            file.sync_all()?;
        }

        // Write revocation cert (recipient.rev) - Can be used to revoke the certificate, shockingly
        {
            let path = temp_dir.join("recipient.rev");
            let mut file = create_owned_file(&path)?;

            let mut armor = Writer::new(&mut file, Kind::Signature)?;

            sequoia_openpgp::serialize::Marshal::serialize(&revocation, &mut armor)?;

            armor.finalize()?;
            file.sync_all()?;
        }

        // Everything works, move to real dir
        fs::rename(&temp_dir, &recipient_dir).context("Failed to move recipient certificate")?;

        #[cfg(unix)]
        File::open(&self.vault_path)?.sync_all()?;

        // TODO: binbows

        Ok(())
    }
}

/// Loads and validates the public and private vault certs.
pub fn load_vault_certs(vault_path: &Path) -> Result<(Cert, Cert)> {
    let recipient_dir = vault_path.join("recipient");

    let public_path = recipient_dir.join("recipient.asc");
    let private_path = recipient_dir.join("recipient.key");

    // Reject symlinks and non-regular files.
    for path in [&public_path, &private_path] {
        let metadata = fs::symlink_metadata(path)
            .with_context(|| format!("Failed to inspect certificate: {}", path.display()))?;

        if !metadata.file_type().is_file() {
            bail!("Certificate is not a regular file: {}", path.display());
        }
    }

    let public_cert = Cert::from_file(&public_path).context("Failed to load public certificate")?;
    let secret_cert =
        Cert::from_file(&private_path).context("Failed to load private certificate")?;
    if public_cert != secret_cert.clone().strip_secret_key_material() {
        bail!("Public and private certificate contents do not match");
    }
    if public_cert.is_tsk() {
        bail!("Public certificate unexpectedly contains secret material");
    }
    if !secret_cert.is_tsk() {
        bail!("Private certificate contains no secret material");
    }
    // The two certificates must represent the same OpenPGP identity.
    if public_cert.fingerprint() != secret_cert.fingerprint() {
        bail!("Public and private certificate fingerprints do not match");
    }

    let policy = StandardPolicy::new();

    // Check for at least one usable signing key.
    let has_signing_key = secret_cert
        .keys()
        .secret()
        .with_policy(&policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .next()
        .is_some();

    if !has_signing_key {
        bail!("Vault has no usable private signing key");
    }

    // Check for at least one usable storage encryption key.
    let has_encryption_key = secret_cert
        .keys()
        .secret()
        .with_policy(&policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_storage_encryption()
        .next()
        .is_some();

    if !has_encryption_key {
        bail!("Vault has no usable private storage encryption key");
    }

    // All secret key material should be encrypted when unused.
    for key in secret_cert.keys().secret() {
        if !key.key().secret().is_encrypted() {
            bail!("Vault contains unencrypted private key material");
        }
    }

    Ok((public_cert, secret_cert))
}

// TODO: Are these repeted code? Yes. Am i a procrastonator who cba merging them? Yea
/// Creates a dir where only the owner has r/w/e permissions
fn create_owned_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;

    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_dir() {
        bail!("Expected a real directory, recieved \'{}\'", path.display());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }

    // TODO: Implement for windows

    Ok(())
}

fn create_owned_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    // TODO: Implement for windows

    Ok(options.open(path)?)
}

/// Unlocks the vault's signing key using the vault passphrase.
pub fn unlock_signing_keypair(secret_cert: &Cert, passphrase: &str) -> Result<KeyPair> {
    if passphrase.is_empty() {
        bail!("Vault passphrase cannot be empty");
    }

    let policy = StandardPolicy::new();
    let password = Password::from(passphrase);

    let signing_key = secret_cert
        .keys()
        .secret()
        .with_policy(&policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .next()
        .context("No usable private signing key found")?;

    let unlocked_key = signing_key
        .key()
        .clone()
        .decrypt_secret(&password)
        .context("Failed to unlock vault signing key")?;

    let keypair = unlocked_key
        .into_keypair()
        .context("Failed to create signing keypair")?;

    Ok(keypair)
}

/// Unlocks every usable storage-encryption key in a vault
pub fn unlock_storage_keypairs(secret_cert: &Cert, passphrase: &str) -> Result<Vec<KeyPair>> {
    if passphrase.is_empty() {
        bail!("Vault passphrase cannot be empty");
    }

    let policy = StandardPolicy::new();
    let password = Password::from(passphrase);

    let mut keypairs = Vec::new();

    for key in secret_cert
        .keys()
        .with_policy(&policy, None)
        .supported()
        .alive()
        .revoked(false)
        .secret()
        .for_storage_encryption()
    {
        let unlocked_key = key
            .key()
            .clone()
            .decrypt_secret(&password)
            .context("Failed to unlock a storage encryption key")?;

        let keypair = unlocked_key
            .into_keypair()
            .context("Failed to create storage decryption keypair")?;

        keypairs.push(keypair);
    }

    if keypairs.is_empty() {
        bail!("Vault has no usable storage decryption keys");
    }

    Ok(keypairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    const TEST_PASSPHRASE: &str = "test-passphrase";

    #[test]
    fn test_recipient_generation() {
        let dir = tempdir().unwrap();

        let builder = RecipientBuilder::new("Test <test@example.org>", dir.path());

        builder.write(TEST_PASSPHRASE).unwrap();

        let recipient_dir = dir.path().join("recipient");

        assert!(recipient_dir.join("recipient.asc").exists());
        assert!(recipient_dir.join("recipient.key").exists());
        assert!(recipient_dir.join("recipient.rev").exists());

        let (public_cert, secret_cert) = load_vault_certs(dir.path()).unwrap();

        assert!(!public_cert.is_tsk());
        assert!(secret_cert.is_tsk());

        assert_eq!(public_cert.fingerprint(), secret_cert.fingerprint());
    }

    #[test]
    fn test_private_key_is_passphrase_protected() {
        let dir = tempdir().unwrap();

        let builder = RecipientBuilder::new("Test <test@example.org>", dir.path());

        builder.write(TEST_PASSPHRASE).unwrap();

        let (_, secret_cert) = load_vault_certs(dir.path()).unwrap();

        assert!(
            secret_cert
                .keys()
                .secret()
                .all(|key| key.key().secret().is_encrypted())
        );

        // Correct passphrase must work.
        assert!(unlock_signing_keypair(&secret_cert, TEST_PASSPHRASE).is_ok());

        // Incorrect passphrase must fail.
        assert!(unlock_signing_keypair(&secret_cert, "definitely-the-wrong-password").is_err());
    }

    #[test]
    fn test_recipient_is_not_overwritten() {
        let dir = tempdir().unwrap();

        let builder = RecipientBuilder::new("Test <test@example.org>", dir.path());

        builder.write(TEST_PASSPHRASE).unwrap();

        assert!(builder.write(TEST_PASSPHRASE).is_err());
    }

    #[test]
    fn test_storage_encryption_key_unlocks() {
        let dir = tempdir().unwrap();

        let builder = RecipientBuilder::new("Test <test@example.org>", dir.path());

        builder.write(TEST_PASSPHRASE).unwrap();

        let (_, secret_cert) = load_vault_certs(dir.path()).unwrap();

        let policy = StandardPolicy::new();
        let password = Password::from(TEST_PASSPHRASE);

        let encryption_key = secret_cert
            .keys()
            .with_policy(&policy, None)
            .supported()
            .alive()
            .revoked(false)
            .secret()
            .for_storage_encryption()
            .next()
            .expect("No storage encryption key found");

        let unlocked = encryption_key
            .key()
            .clone()
            .decrypt_secret(&password)
            .expect("Failed to unlock storage encryption key");

        let keypair = unlocked
            .into_keypair()
            .expect("Failed to construct encryption keypair");

        assert!(
            keypair.public().pk_algo().is_supported(),
            "Encryption algorithm is unsupported"
        );
    }
}
