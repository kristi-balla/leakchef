use std::{collections::HashMap, fmt::Display};

use bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Default, Serialize, Deserialize, Clone, TypedBuilder)]
#[builder(field_defaults(default, setter(into)))]
pub struct Identity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub leak_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub linenumber: Option<isize>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub email: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub phone: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub password: Vec<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub hash: HashMap<HashType, Vec<String>>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cc: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub iban: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub domain: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub blz: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub user: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ip: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub date: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub unknown: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashType {
    #[serde(rename = "md5")]
    Md5,

    #[serde(rename = "sha1")]
    Sha1,

    #[serde(rename = "sha23")]
    Sha23,

    #[serde(rename = "mysql")]
    MySql,

    #[serde(rename = "bcrypt")]
    Bcrypt,

    #[serde(rename = "phpbb3")]
    PhpBb3,

    #[serde(rename = "mcf")]
    Mcf,

    #[serde(rename = "phc")]
    Phc,

    #[default]
    #[serde(other, rename = "unknown")]
    Unknown,
}

impl Display for HashType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashType::Md5 => write!(f, "md5"),
            HashType::Sha1 => write!(f, "sha1"),
            HashType::Sha23 => write!(f, "sha23"),
            HashType::MySql => write!(f, "mysql"),
            HashType::Bcrypt => write!(f, "bcrypt"),
            HashType::PhpBb3 => write!(f, "phpbb3"),
            HashType::Mcf => write!(f, "mcf"),
            HashType::Phc => write!(f, "phc"),
            HashType::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<HashType> for Bson {
    fn from(hash_type: HashType) -> Self {
        Bson::String(hash_type.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TypedBuilder)]
pub struct PartialIdentity {
    #[serde(rename = "_id")]
    pub object_id: ObjectId,

    #[serde(rename = "email", skip_serializing_if = "Vec::is_empty", default)]
    pub emails: Vec<String>,

    #[serde(rename = "phone", skip_serializing_if = "Vec::is_empty", default)]
    pub phones: Vec<String>,

    #[serde(rename = "domain", skip_serializing_if = "Vec::is_empty", default)]
    pub domains: Vec<String>,

    #[serde(rename = "password", skip_serializing_if = "Vec::is_empty", default)]
    pub passwords: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdPasswordPair {
    pub id: String,
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MappedIdentity {
    /// The original MongoDB ObjectId of the unprocessed identity.
    #[serde(rename = "_id")]
    pub object_id: ObjectId,

    /// A list of (identifier, password) pairs.
    #[serde(rename = "credentials", default, skip_serializing_if = "Vec::is_empty")]
    pub credentials: Vec<IdPasswordPair>,
}

impl From<PartialIdentity> for MappedIdentity {
    fn from(mut id: PartialIdentity) -> Self {
        let identifiers = match (id.emails.len(), id.phones.len()) {
            // No identifiers left, return early
            (0, 0) => {
                return Self {
                    object_id: id.object_id,
                    credentials: Vec::new(),
                }
            }
            // Only emails relevant, reuse the email vec
            (_n, 0) => id.emails,
            // Only phones relevant, reuse the phone vec
            (0, _n) => id.phones,
            // Both emails and phones are relevant, extend the email vec
            (_n, _m) => {
                id.emails.extend(id.phones);
                id.emails
            }
        };

        let mut credentials = Vec::with_capacity(id.passwords.len());

        for pw in id.passwords {
            match pw.split_once(':') {
                // Map passwords with prefix to every identifier matching it
                Some((id_prefix, real_pw)) => {
                    for id in identifiers.iter().filter(|id| !id.starts_with(id_prefix)) {
                        credentials.push(IdPasswordPair {
                            id: id.clone(),
                            password: real_pw.to_string(),
                        });
                    }
                }
                None => match identifiers.len() {
                    // Single identifiers can be mapped to the password directly.
                    // This should be the most common case so it helps avoiding a clone here.
                    1 => credentials.push(IdPasswordPair {
                        id: identifiers[0].clone(),
                        password: pw,
                    }),
                    // The password does not have a prefix and there are multiple identifiers.
                    _ => {
                        for id in &identifiers {
                            credentials.push(IdPasswordPair {
                                id: id.clone(),
                                password: pw.clone(),
                            })
                        }
                    }
                },
            }
        }

        Self {
            object_id: id.object_id,
            credentials,
        }
    }
}

impl From<Identity> for PartialIdentity {
    fn from(value: Identity) -> Self {
        let object_id = match value.id {
            Some(id) => id,
            None => ObjectId::new(),
        };

        let emails = value.email;
        let phones = value.phone;
        let passwords = value.password;
        let domains = value.domain;

        PartialIdentity::builder()
            .emails(emails)
            .phones(phones)
            .domains(domains)
            .passwords(passwords)
            .object_id(object_id)
            .build()
    }
}
