//! Encryption of off-site backups with [age](https://age-encryption.org).
//!
//! The shop only knows the owner's **public** key (`age1…`), so it can encrypt backups but never read
//! them. The private key (`AGE-SECRET-KEY-1…`) stays offline with the owner; for a restore it is entered
//! in the admin, used in memory for the decryption and dropped (zeroized) right after. It is never stored
//! or logged.

use std::io::{Read, Write};
use std::path::Path;
use zeroize::Zeroizing;

/// First line of every age file (binary format).
const AGE_MAGIC: &[u8] = b"age-encryption.org/v1\n";
/// Encrypted backups get this suffix after `.zip`.
pub const ENCRYPTED_SUFFIX: &str = ".age";

/// A private key entered by the admin. Zeroized when dropped; deliberately without `Debug`/`Display`.
pub struct SecretIdentity(Zeroizing<String>);

impl SecretIdentity {
    pub fn new(text: String) -> Self {
        SecretIdentity(Zeroizing::new(text))
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }

    /// The `AGE-SECRET-KEY-1…` line of a pasted key or an identity file (`age-keygen` output with comments).
    fn parse(&self) -> Result<age::x25519::Identity, String> {
        let line = self
            .0
            .lines()
            .map(str::trim)
            .find(|l| l.starts_with("AGE-SECRET-KEY-1"))
            .ok_or("This is not an age private key. It starts with AGE-SECRET-KEY-1 (the line from the key file, not the age1… public key).")?;
        line.parse::<age::x25519::Identity>()
            .map_err(|_| "The private key is damaged or incomplete. Copy the whole AGE-SECRET-KEY-1… line.".to_string())
    }
}

/// Checks and normalises a public key entered in the settings.
pub fn normalize_recipient(input: &str) -> Result<String, String> {
    let key = input.trim();
    if key.to_uppercase().starts_with("AGE-SECRET-KEY") {
        return Err("This is the PRIVATE key. Never enter it into the shop — enter the public key (age1…) and keep the private key offline.".to_string());
    }
    if !key.starts_with("age1") {
        return Err("Enter an age public key — it starts with age1… (age-keygen prints it as \"Public key: age1…\").".to_string());
    }
    key.parse::<age::x25519::Recipient>()
        .map_err(|_| "This age public key is not valid. Copy the whole age1… key.".to_string())?;
    Ok(key.to_string())
}

/// True when the file is age-encrypted (whatever its name).
pub fn is_encrypted(path: &Path) -> bool {
    let mut head = [0u8; AGE_MAGIC.len()];
    std::fs::File::open(path).and_then(|mut f| f.read_exact(&mut head)).is_ok() && head == AGE_MAGIC
}

/// Encrypts `src` to `recipient` into `dst` (streaming, any size).
pub fn encrypt_file(src: &Path, dst: &Path, recipient: &str) -> Result<(), String> {
    let recipient: age::x25519::Recipient = recipient.parse().map_err(|_| "The saved age public key is not valid.".to_string())?;
    let encryptor = age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
        .map_err(|e| format!("Encrypting the backup failed: {}", e))?;
    let mut input = std::io::BufReader::new(std::fs::File::open(src).map_err(|e| e.to_string())?);
    let output = std::io::BufWriter::new(std::fs::File::create(dst).map_err(|e| e.to_string())?);
    let mut writer = encryptor.wrap_output(output).map_err(|e| format!("Encrypting the backup failed: {}", e))?;
    std::io::copy(&mut input, &mut writer).map_err(|e| format!("Encrypting the backup failed: {}", e))?;
    writer
        .finish()
        .and_then(|mut w| w.flush())
        .map_err(|e| format!("Encrypting the backup failed: {}", e))
}

/// Decrypts an age file with the admin's private key into `dst`.
pub fn decrypt_file(src: &Path, dst: &Path, identity: &SecretIdentity) -> Result<(), String> {
    let identity = identity.parse()?;
    let input = std::io::BufReader::new(std::fs::File::open(src).map_err(|e| e.to_string())?);
    let decryptor = age::Decryptor::new(input).map_err(|_| "The encrypted backup is damaged.".to_string())?;
    if decryptor.is_scrypt() {
        return Err("This backup is protected with a passphrase, not with a key. Decrypt it with `age -d` first and upload the ZIP.".to_string());
    }
    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(|_| "This private key does not belong to the public key the backup was encrypted with.".to_string())?;
    let mut output = std::io::BufWriter::new(std::fs::File::create(dst).map_err(|e| e.to_string())?);
    std::io::copy(&mut reader, &mut output).map_err(|_| "The encrypted backup is damaged or incomplete.".to_string())?;
    output.flush().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::ExposeSecret;

    fn dir() -> crate::services::backup::TempDir {
        let d = std::env::temp_dir().join(format!("age-test-{}", crate::services::auth::random_token(6)));
        std::fs::create_dir_all(&d).unwrap();
        crate::services::backup::TempDir(d)
    }

    #[test]
    fn encrypts_for_the_public_key_and_decrypts_with_the_private_key() {
        let d = dir();
        let key = age::x25519::Identity::generate();
        let public = key.to_public().to_string();
        let identity_file = format!("# created: 2026-11-01\n# public key: {}\n{}\n", public, key.to_string().expose_secret());
        std::fs::write(d.0.join("plain.zip"), b"PK\x03\x04 customer data").unwrap();

        encrypt_file(&d.0.join("plain.zip"), &d.0.join("b.zip.age"), &public).unwrap();
        let encrypted = std::fs::read(d.0.join("b.zip.age")).unwrap();
        assert!(is_encrypted(&d.0.join("b.zip.age")) && !is_encrypted(&d.0.join("plain.zip")));
        assert!(!encrypted.windows(13).any(|w| w == b"customer data"));

        decrypt_file(&d.0.join("b.zip.age"), &d.0.join("out.zip"), &SecretIdentity::new(identity_file)).unwrap();
        assert_eq!(std::fs::read(d.0.join("out.zip")).unwrap(), b"PK\x03\x04 customer data");

        let other = age::x25519::Identity::generate();
        let err = decrypt_file(&d.0.join("b.zip.age"), &d.0.join("x.zip"), &SecretIdentity::new(other.to_string().expose_secret().to_string()));
        assert!(err.unwrap_err().contains("does not belong"));
        assert!(decrypt_file(&d.0.join("b.zip.age"), &d.0.join("x.zip"), &SecretIdentity::new("age1abc".into())).is_err());
    }

    #[test]
    fn only_valid_public_keys_are_accepted() {
        let key = age::x25519::Identity::generate();
        let public = key.to_public().to_string();
        assert_eq!(normalize_recipient(&format!("  {}\n", public)).unwrap(), public);
        assert!(normalize_recipient(key.to_string().expose_secret()).unwrap_err().contains("PRIVATE"));
        assert!(normalize_recipient("ssh-ed25519 AAAA").is_err());
        assert!(normalize_recipient("age1notavalidkey").is_err());
    }
}
