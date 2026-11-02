//! Encryption of off-site backups with [age](https://age-encryption.org), like HarmonicFlow's backups.
//!
//! The shop only knows the owner's **public** keys (`age1…`), so it can encrypt backups but never read them: a leaked
//! GitHub token or a hacked GitHub account exposes nothing readable. Backups are only ever uploaded encrypted; without
//! a public key no backup leaves the server. Several public keys may be set (e.g. a second emergency key), and any of
//! the matching private keys opens the backups.
//!
//! The private key (`AGE-SECRET-KEY-1…`) stays offline with the owner. For a restore it is entered in the admin, used
//! in memory for the decryption and dropped (zeroized) right after. It is never stored or logged.

use std::io::{Read, Write};
use std::path::Path;
use zeroize::Zeroizing;

/// First line of every age file (binary format).
const AGE_MAGIC: &[u8] = b"age-encryption.org/v1\n";
/// Encrypted backups get this suffix after `.zip`.
pub const ENCRYPTED_SUFFIX: &str = ".age";
/// More keys than this is certainly a paste mistake.
const MAX_RECIPIENTS: usize = 10;

/// A private key entered by the admin. Zeroized when dropped; deliberately without `Debug`/`Display`.
pub struct SecretIdentity(Zeroizing<String>);

impl SecretIdentity {
    pub fn new(text: String) -> Self {
        SecretIdentity(Zeroizing::new(text))
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }

    /// The private key(s) of a pasted `AGE-SECRET-KEY-1…` line or a whole key file (`age-keygen` output with comments).
    fn parse(&self) -> Result<Vec<Box<dyn age::Identity + Send + Sync>>, String> {
        if !self.0.contains("AGE-SECRET-KEY-1") {
            return Err("This is not an age private key. It starts with AGE-SECRET-KEY-1 — use the key file from age-keygen, not the age1… public key.".to_string());
        }
        age::IdentityFile::from_buffer(std::io::Cursor::new(self.0.as_bytes()))
            .and_then(|f| f.into_identities().map_err(|e| std::io::Error::other(e.to_string())))
            .map_err(|_| "The private key is damaged or incomplete. Copy the whole AGE-SECRET-KEY-1… line or choose the key file.".to_string())
    }
}

/// True when the text contains a private key — it must never be stored, so the settings refuse it.
pub fn contains_private_key(text: &str) -> bool {
    text.to_uppercase().contains("AGE-SECRET-KEY-")
}

/// Picks the public keys out of what an admin pastes: bare `age1…` keys, age-keygen's `Public key: age1…` line or the
/// `# public key: age1…` comment of a key file — one or several, separated by lines, spaces or commas.
/// Returns them one per line (the stored form). Empty input gives an empty string (= no encryption key).
pub fn normalize_recipients(input: &str) -> Result<String, String> {
    if contains_private_key(input) {
        return Err("This contains your PRIVATE key (AGE-SECRET-KEY-…). It was not saved. Never enter it here — enter only the public key (age1…) and keep the key file offline.".to_string());
    }
    let mut keys: Vec<String> = Vec::new();
    for word in input.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let word = word.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        if !word.to_lowercase().starts_with("age1") {
            continue;
        }
        let key = word.to_lowercase();
        key.parse::<age::x25519::Recipient>()
            .map_err(|_| format!("\"{}…\" is not a valid age public key. Copy the whole age1… key.", &key[..key.len().min(12)]))?;
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    if keys.is_empty() && !input.trim().is_empty() {
        return Err("No age public key found. It starts with age1… (age-keygen prints it as \"Public key: age1…\").".to_string());
    }
    if keys.len() > MAX_RECIPIENTS {
        return Err(format!("At most {} public keys can be used.", MAX_RECIPIENTS));
    }
    Ok(keys.join("\n"))
}

/// The stored keys as age recipients.
fn recipients(stored: &str) -> Result<Vec<age::x25519::Recipient>, String> {
    let keys: Vec<age::x25519::Recipient> = stored
        .split_whitespace()
        .map(|k| k.parse::<age::x25519::Recipient>().map_err(|_| "A saved age public key is not valid.".to_string()))
        .collect::<Result<_, _>>()?;
    if keys.is_empty() {
        return Err("No age public key is set: backups are only uploaded encrypted. Enter your public key (age1…) in Settings → Export & Backups.".to_string());
    }
    Ok(keys)
}

/// How many public keys are stored.
pub fn recipient_count(stored: &str) -> usize {
    stored.split_whitespace().count()
}

/// True when the file is age-encrypted (whatever its name).
pub fn is_encrypted(path: &Path) -> bool {
    let mut head = [0u8; AGE_MAGIC.len()];
    std::fs::File::open(path).and_then(|mut f| f.read_exact(&mut head)).is_ok() && head == AGE_MAGIC
}

fn encryptor(stored: &str) -> Result<age::Encryptor, String> {
    let keys = recipients(stored)?;
    age::Encryptor::with_recipients(keys.iter().map(|r| r as &dyn age::Recipient)).map_err(|e| format!("Encrypting failed: {}", e))
}

/// Encrypts `src` to every stored public key into `dst` (streaming, any size).
pub fn encrypt_file(src: &Path, dst: &Path, stored_recipients: &str) -> Result<(), String> {
    let fail = |e: std::io::Error| format!("Encrypting the backup failed: {}", e);
    let encryptor = encryptor(stored_recipients)?;
    let mut input = std::io::BufReader::new(std::fs::File::open(src).map_err(fail)?);
    let output = std::io::BufWriter::new(std::fs::File::create(dst).map_err(fail)?);
    let mut writer = encryptor.wrap_output(output).map_err(fail)?;
    std::io::copy(&mut input, &mut writer).map_err(fail)?;
    writer.finish().and_then(|mut w| w.flush()).map_err(fail)
}

/// A small encrypted text file the owner opens with their private key to prove the key pair works.
pub fn encrypt_test_file(stored_recipients: &str, store_name: &str, created: &str) -> Result<Vec<u8>, String> {
    let fail = |e: std::io::Error| format!("Encrypting the test file failed: {}", e);
    let text = format!(
        "It works!\n\nYour private key opens the backups of {}.\nThis test file was encrypted on {} to:\n{}\n\n\
         Keep the key file in your password manager and in a second place that is not this server.\n",
        if store_name.is_empty() { "your shop" } else { store_name },
        created,
        stored_recipients
    );
    let mut out = Vec::new();
    let mut writer = encryptor(stored_recipients)?.wrap_output(&mut out).map_err(fail)?;
    writer.write_all(text.as_bytes()).map_err(fail)?;
    writer.finish().map_err(fail)?;
    Ok(out)
}

/// Decrypts an age file with the admin's private key into `dst`.
pub fn decrypt_file(src: &Path, dst: &Path, identity: &SecretIdentity) -> Result<(), String> {
    let identities = identity.parse()?;
    let input = std::io::BufReader::new(std::fs::File::open(src).map_err(|e| e.to_string())?);
    let decryptor = age::Decryptor::new_buffered(input).map_err(|_| "The encrypted backup is damaged.".to_string())?;
    let mut reader = decryptor
        .decrypt(identities.iter().map(|i| i.as_ref() as &dyn age::Identity))
        .map_err(|_| "This private key does not belong to a public key the backup was encrypted with.".to_string())?;
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

    fn key_file(key: &age::x25519::Identity) -> String {
        format!("# created: 2026-11-02T10:00:00+01:00\n# public key: {}\n{}\n", key.to_public(), key.to_string().expose_secret())
    }

    #[test]
    fn every_listed_key_opens_the_backup() {
        let d = dir();
        let (main, emergency, stranger) = (age::x25519::Identity::generate(), age::x25519::Identity::generate(), age::x25519::Identity::generate());
        let stored = normalize_recipients(&format!("{}\n{}", main.to_public(), emergency.to_public())).unwrap();
        std::fs::write(d.0.join("plain.zip"), b"PK\x03\x04 customer data").unwrap();

        encrypt_file(&d.0.join("plain.zip"), &d.0.join("b.zip.age"), &stored).unwrap();
        assert!(is_encrypted(&d.0.join("b.zip.age")) && !is_encrypted(&d.0.join("plain.zip")));
        assert!(!std::fs::read(d.0.join("b.zip.age")).unwrap().windows(13).any(|w| w == b"customer data"));

        for key in [&main, &emergency] {
            decrypt_file(&d.0.join("b.zip.age"), &d.0.join("out.zip"), &SecretIdentity::new(key_file(key))).unwrap();
            assert_eq!(std::fs::read(d.0.join("out.zip")).unwrap(), b"PK\x03\x04 customer data");
        }
        let err = decrypt_file(&d.0.join("b.zip.age"), &d.0.join("x.zip"), &SecretIdentity::new(key_file(&stranger))).unwrap_err();
        assert!(err.contains("does not belong"));
        assert!(decrypt_file(&d.0.join("b.zip.age"), &d.0.join("x.zip"), &SecretIdentity::new(main.to_public().to_string())).is_err());
    }

    #[test]
    fn public_keys_are_picked_out_of_what_people_paste() {
        let (a, b) = (age::x25519::Identity::generate(), age::x25519::Identity::generate());
        let (pa, pb) = (a.to_public().to_string(), b.to_public().to_string());
        assert_eq!(normalize_recipients(&format!("  {}\n", pa)).unwrap(), pa);
        assert_eq!(normalize_recipients(&format!("Public key: {}", pa)).unwrap(), pa);
        assert_eq!(normalize_recipients(&format!("# public key: {}", pa)).unwrap(), pa);
        assert_eq!(normalize_recipients(&format!("{}, {}\n{}", pa, pb, pa)).unwrap(), format!("{}\n{}", pa, pb));
        assert_eq!(normalize_recipients("").unwrap(), "");
        assert_eq!(recipient_count(&format!("{}\n{}", pa, pb)), 2);
        // A whole key file or the private key itself is never accepted
        assert!(normalize_recipients(&key_file(&a)).unwrap_err().contains("PRIVATE"));
        assert!(normalize_recipients(a.to_string().expose_secret()).unwrap_err().contains("PRIVATE"));
        assert!(normalize_recipients("ssh-ed25519 AAAA").is_err());
        assert!(normalize_recipients("age1notavalidkey").is_err());
    }

    #[test]
    fn nothing_is_encrypted_without_a_key_and_the_test_file_opens() {
        assert!(encrypt_file(Path::new("x"), Path::new("y"), "").unwrap_err().contains("only uploaded encrypted"));
        let key = age::x25519::Identity::generate();
        let file = encrypt_test_file(&key.to_public().to_string(), "Example Store", "2026-11-02").unwrap();
        let d = dir();
        std::fs::write(d.0.join("t.age"), &file).unwrap();
        decrypt_file(&d.0.join("t.age"), &d.0.join("t.txt"), &SecretIdentity::new(key_file(&key))).unwrap();
        assert!(std::fs::read_to_string(d.0.join("t.txt")).unwrap().starts_with("It works!"));
    }
}

/// Interoperability with the official `age` tool: run with AGE_INTEROP_DIR=<dir> containing `key.txt`
/// (made by `age-keygen`) and `from-cli.age` (made by `age -r … `); writes `from-shop.age` for `age -d`.
#[cfg(test)]
#[test]
fn interop_with_the_age_cli() {
    let Ok(dir) = std::env::var("AGE_INTEROP_DIR") else { return };
    let dir = Path::new(&dir);
    let key_file = std::fs::read_to_string(dir.join("key.txt")).unwrap();
    let public = normalize_recipients(key_file.lines().find(|l| l.contains("public key")).unwrap()).unwrap();
    std::fs::write(dir.join("from-shop.age"), encrypt_test_file(&public, "Example Store", "2026-11-02").unwrap()).unwrap();
    decrypt_file(&dir.join("from-cli.age"), &dir.join("from-cli.txt"), &SecretIdentity::new(key_file)).unwrap();
}
