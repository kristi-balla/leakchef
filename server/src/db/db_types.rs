use std::fmt::{Display, Formatter};

use bson::{oid::ObjectId, Bson, doc};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use typed_builder::TypedBuilder;

pub type MetadataId = ObjectId;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[builder(field_defaults(default, setter(into)))]
pub struct Metadata {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<MetadataId>,

    pub leak_id: String,

    pub parser: Option<String>,

    pub file_name: Option<String>,

    #[serde(rename = "filepath")]
    pub file_path: Option<String>,

    pub date_parsed: Option<isize>,

    #[serde(rename = "file_size")]
    pub size: Option<isize>,

    #[serde(rename = "file_line_count")]
    pub num_lines: Option<isize>,

    #[serde(rename = "parsed_identities")]
    pub extracted_identities: isize,

    #[serde(default = "get_null_usize")]
    pub already_read_lines: usize,

    #[serde(default)]
    pub status: LeakStatus,

    #[serde(default)]
    pub file_type: FileType,

    #[serde(
        rename = "detected_fields",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub extracted_types: Vec<FieldType>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub leak_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub leak_source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub date_collected: Option<isize>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub date_published: Option<isize>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub date_approx_leaked: Option<isize>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub internal_information: Option<Value>,

    #[serde(skip_serializing_if = "ExtendedInformation::is_empty", default)]
    pub extended_information: ExtendedInformation,
}

fn get_null_usize() -> usize {
    0
}

/// Enum denoting the possible types a specific field or column in a leak can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FieldType {
    #[serde(rename = "password")]
    Password,

    #[serde(rename = "email")]
    Email,

    #[serde(rename = "domain")]
    Domain,

    #[serde(rename = "username")]
    Username,

    #[serde(rename = "hash-md5")]
    HashMd5,

    #[serde(rename = "hash-sha1")]
    HashSha1,

    #[serde(rename = "hash-sha23")]
    HashSha23,

    #[serde(rename = "hash-mysql")]
    HashMySql,

    #[serde(rename = "hash-bcrypt")]
    HashBcrypt,

    #[serde(rename = "hash-phpbb3")]
    HashPhpBb3,

    #[serde(rename = "hash-mcf")]
    HashMcf,

    #[serde(rename = "hash-phc")]
    HashPhc,

    #[serde(rename = "blz")]
    Blz,

    #[serde(rename = "iban")]
    Iban,

    #[serde(rename = "ip")]
    Ip,

    #[serde(rename = "cc")]
    CreditCard,

    #[serde(rename = "phone")]
    Phone,

    #[serde(rename = "date")]
    Date,

    #[serde(rename = "timestamp")]
    TimeStamp,

    #[serde(rename = "unknown", other)]
    #[default]
    Unknown,
}

impl Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Password => write!(f, "password"),
            FieldType::Email => write!(f, "email"),
            FieldType::Domain => write!(f, "domain"),
            FieldType::Username => write!(f, "username"),
            FieldType::Blz => write!(f, "blz"),
            FieldType::Iban => write!(f, "iban"),
            FieldType::Ip => write!(f, "ip"),
            FieldType::CreditCard => write!(f, "cc"),
            FieldType::Phone => write!(f, "phone"),
            FieldType::Date => write!(f, "date"),
            FieldType::Unknown => write!(f, "unknown"),
            FieldType::HashMd5 => write!(f, "hash-md5"),
            FieldType::HashSha1 => write!(f, "hash-sha1"),
            FieldType::HashSha23 => write!(f, "hash-sha23"),
            FieldType::HashMySql => write!(f, "hash-mysql"),
            FieldType::HashBcrypt => write!(f, "hash-bcrypt"),
            FieldType::HashPhpBb3 => write!(f, "hash-phpbb3"),
            FieldType::HashMcf => write!(f, "hash-mcf"),
            FieldType::HashPhc => write!(f, "hash-phc"),
            FieldType::TimeStamp => write!(f, "timestamp"),
        }
    }
}

impl From<FieldType> for Bson {
    fn from(state: FieldType) -> Self {
        Bson::String(state.to_string())
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeakStatus {
    #[serde(rename = "new")]
    New,

    #[serde(rename = "in-progress")]
    InProgress,

    #[serde(rename = "failed")]
    Failed,

    #[serde(rename = "finished")]
    Finished,

    /// Indicates that the leak was manually disabled and should not be used.
    ///
    /// This is meant as a means to filter out a subset of leaks for testing or
    /// benchmarking without having to recreate an entire database or modify
    /// applications like the live-feeder.
    #[serde(rename = "disabled")]
    Disabled,

    #[default]
    #[serde(other, rename = "unknown")]
    Unknown,
}

impl Display for LeakStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeakStatus::New => write!(f, "new"),
            LeakStatus::InProgress => write!(f, "in-progress"),
            LeakStatus::Failed => write!(f, "failed"),
            LeakStatus::Finished => write!(f, "finished"),
            LeakStatus::Disabled => write!(f, "disabled"),
            LeakStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<LeakStatus> for Bson {
    fn from(state: LeakStatus) -> Self {
        Bson::String(state.to_string())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(default, setter(into)))]
pub struct ExtendedInformation {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub leak_urls: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub leak_sources: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub file_paths: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub file_names: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dates_collected: Vec<isize>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dates_published: Vec<isize>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dates_approx_leaked: Vec<isize>,
}

impl ExtendedInformation {
    pub fn is_empty(&self) -> bool {
        self.leak_urls.is_empty()
            && self.leak_sources.is_empty()
            && self.file_paths.is_empty()
            && self.file_names.is_empty()
            && self.dates_collected.is_empty()
            && self.dates_published.is_empty()
            && self.dates_approx_leaked.is_empty()
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileType {
    #[serde(rename = "dsv")]
    Dsv,

    #[serde(rename = "sql")]
    Sql,

    #[default]
    #[serde(other, rename = "unknown")]
    Unknown,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Dsv => write!(f, "dsv"),
            FileType::Sql => write!(f, "sql"),
            FileType::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<FileType> for Bson {
    fn from(state: FileType) -> Self {
        Bson::String(state.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub customer_id: i32,
    pub handled_leaks: Vec<String>,
    pub customer_salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct Status {
    pub customer_id: i32,
    pub current_leak_id: String,
    pub identities_left: u32,

    #[builder(default)]
    pub last_received_identity: Option<ObjectId>,

    #[builder(default)]
    pub leak_status: Option<LeakStatus>,

    #[builder(default)]
    pub leak_result: Option<LeakResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TypedBuilder)]
pub struct LeakResult {
    pub identities_received: u32,
    pub full_matches: i32,
}

impl Display for LeakResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"identities_received\": {}, \"number_of_matches\": {}}}",
            self.identities_received, self.full_matches
        )
    }
}

impl From<LeakResult> for Bson {
    fn from(state: LeakResult) -> Self {
        let result = doc!{
            "identities_received": state.identities_received,
            "full_matches": state.full_matches,
        };
        Bson::Document(result)
    }
}
