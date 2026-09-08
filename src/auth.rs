//! Operator-provisioned bearer credentials. Never serialize or log secrets.

use crate::config::validate_id;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::RwLock;
use subtle::ConstantTimeEq;

const MAX_VERIFIER_BYTES: usize = 65_536;
const MAX_CREDENTIALS: usize = 256;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialRecord {
    pub token_id: String,
    pub owner_id: String,
    pub secret_sha256: String,
    pub expires_unix_s: u64,
    pub revoked: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifierFile {
    pub version: u32,
    pub credentials: Vec<CredentialRecord>,
}

struct Verifier {
    owner: String,
    digest: [u8; 32],
    expires: u64,
    revoked: bool,
}

pub struct Credentials {
    records: RwLock<BTreeMap<String, Verifier>>,
}

impl Credentials {
    /// Trusted startup/reload only. The configuration validator keeps this path
    /// outside all tool roots; the operator must provision private permissions.
    pub fn load(path: &Path, owners: &BTreeSet<String>) -> Result<Self, String> {
        let file = File::open(path).map_err(|_| "cannot_open_credential_verifiers")?;
        if !file
            .metadata()
            .map_err(|_| "cannot_inspect_credential_verifiers")?
            .is_file()
        {
            return Err("invalid_credential_verifier_file".into());
        }
        let mut bytes = Vec::new();
        file.take((MAX_VERIFIER_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| "cannot_read_credential_verifiers")?;
        Self::parse(&bytes, owners)
    }

    pub fn parse(bytes: &[u8], owners: &BTreeSet<String>) -> Result<Self, String> {
        if bytes.len() > MAX_VERIFIER_BYTES {
            return Err("credential_verifiers_too_large".into());
        }
        let file: VerifierFile =
            serde_json::from_slice(bytes).map_err(|_| "invalid_credential_verifiers")?;
        if file.version != 1
            || file.credentials.is_empty()
            || file.credentials.len() > MAX_CREDENTIALS
        {
            return Err("invalid_credential_verifiers".into());
        }
        let mut records = BTreeMap::new();
        for record in file.credentials {
            validate_id(&record.token_id)?;
            validate_id(&record.owner_id)?;
            let digest = decode_32(&record.secret_sha256).ok_or("invalid_credential_verifier")?;
            if !owners.contains(&record.owner_id)
                || record.expires_unix_s == 0
                || records
                    .insert(
                        record.token_id,
                        Verifier {
                            owner: record.owner_id,
                            digest,
                            expires: record.expires_unix_s,
                            revoked: record.revoked,
                        },
                    )
                    .is_some()
            {
                return Err("invalid_credential_verifiers".into());
            }
        }
        Ok(Self {
            records: RwLock::new(records),
        })
    }

    /// Only the HTTP layer's one bounded Authorization header reaches here.
    /// Unknown, expired, revoked and malformed credentials have one result.
    pub fn authenticate(&self, authorization: &str, now_unix_s: u64) -> Option<String> {
        if authorization.len() > 160 {
            return None;
        }
        let token = authorization.strip_prefix("Bearer ")?;
        let (id, encoded) = token.split_once('.')?;
        if validate_id(id).is_err() {
            return None;
        }
        let secret = decode_32(encoded)?;
        let digest: [u8; 32] = Sha256::digest(secret).into();
        let records = self.records.read().ok()?;
        let record = records.get(id);
        let expected = record.map_or(&[0_u8; 32], |record| &record.digest);
        let matches = bool::from(digest.ct_eq(expected));
        let record = record?;
        (matches && !record.revoked && now_unix_s < record.expires).then(|| record.owner.clone())
    }

    /// Deliberate operator action; no client-facing revocation route exists.
    /// Already admitted runs retain their frozen authority.
    pub fn revoke(&self, token_id: &str) -> bool {
        let Ok(mut records) = self.records.write() else {
            return false;
        };
        let Some(record) = records.get_mut(token_id) else {
            return false;
        };
        record.revoked = true;
        true
    }

    pub fn replace(&self, replacement: Self) -> Result<(), String> {
        let replacement = replacement
            .records
            .into_inner()
            .map_err(|_| "credential_lock_failed")?;
        *self.records.write().map_err(|_| "credential_lock_failed")? = replacement;
        Ok(())
    }

    pub fn disable_all(&self) {
        if let Ok(mut records) = self.records.write() {
            records.clear();
        }
    }
}

/// This result intentionally has neither Debug nor Serialize. The caller stores
/// only `record` and delivers `take_secret()` once through operator provisioning.
pub struct ProvisionedToken {
    pub record: CredentialRecord,
    token: String,
}

impl ProvisionedToken {
    pub fn take_secret(self) -> String {
        self.token
    }
}

pub fn provision(owner: &str, expires_unix_s: u64) -> Result<ProvisionedToken, String> {
    validate_id(owner)?;
    if expires_unix_s == 0 {
        return Err("invalid_credential_expiry".into());
    }
    let mut entropy = [0_u8; 48];
    getrandom::fill(&mut entropy).map_err(|_| "credential_randomness_failed")?;
    let id = hex(&entropy[..16]);
    let secret = &entropy[16..];
    Ok(ProvisionedToken {
        record: CredentialRecord {
            token_id: id.clone(),
            owner_id: owner.into(),
            secret_sha256: hex(&Sha256::digest(secret)),
            expires_unix_s,
            revoked: false,
        },
        token: format!("{id}.{}", hex(secret)),
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn decode_32(encoded: &str) -> Option<[u8; 32]> {
    if encoded.len() != 64
        || !encoded
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bearer_secret_authenticates_but_verifier_expiry_and_revocation_do_not() {
        let first = provision("alice", 100).unwrap();
        let second = provision("bob", 200).unwrap();
        let id = first.record.token_id.clone();
        let digest = first.record.secret_sha256.clone();
        let bytes = serde_json::to_vec(&VerifierFile {
            version: 1,
            credentials: vec![first.record.clone(), second.record.clone()],
        })
        .unwrap();
        let secret = first.take_secret();
        assert!(
            !String::from_utf8(bytes.clone())
                .unwrap()
                .contains(secret.split_once('.').unwrap().1)
        );
        let credentials =
            Credentials::parse(&bytes, &BTreeSet::from(["alice".into(), "bob".into()])).unwrap();
        assert_eq!(
            credentials
                .authenticate(&format!("Bearer {secret}"), 99)
                .as_deref(),
            Some("alice")
        );
        assert!(
            credentials
                .authenticate(&format!("Bearer {secret}"), 100)
                .is_none()
        );
        assert!(
            credentials
                .authenticate(&format!("Bearer {id}.{digest}"), 99)
                .is_none()
        );
        assert!(
            credentials
                .authenticate(
                    &format!("Bearer unknown.{}", secret.split_once('.').unwrap().1),
                    99
                )
                .is_none()
        );
        assert!(credentials.revoke(&id));
        assert!(
            credentials
                .authenticate(&format!("Bearer {secret}"), 99)
                .is_none()
        );
        assert_eq!(
            credentials
                .authenticate(&format!("Bearer {}", second.take_secret()), 99)
                .as_deref(),
            Some("bob")
        );
    }

    #[test]
    fn malformed_headers_records_and_duplicate_ids_fail_closed() {
        let token = provision("alice", 100).unwrap();
        let file = VerifierFile {
            version: 1,
            credentials: vec![token.record.clone()],
        };
        let owners = BTreeSet::from(["alice".into()]);
        let credentials = Credentials::parse(&serde_json::to_vec(&file).unwrap(), &owners).unwrap();
        for bad in [
            "",
            "Bearer ",
            "Basic x",
            "Bearer x.y",
            "Bearer x.🦀",
            "Bearer x.0000",
            "Bearer x.y.z",
        ] {
            assert!(credentials.authenticate(bad, 1).is_none());
        }
        assert!(Credentials::parse(&vec![b' '; MAX_VERIFIER_BYTES + 1], &owners).is_err());
        assert!(Credentials::parse(&serde_json::to_vec(&file).unwrap(), &BTreeSet::new()).is_err());
        let duplicated = VerifierFile {
            version: 1,
            credentials: vec![token.record.clone(), token.record],
        };
        assert!(Credentials::parse(&serde_json::to_vec(&duplicated).unwrap(), &owners).is_err());
    }
}
