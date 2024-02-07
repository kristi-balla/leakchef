use std::fmt::{self, Display};

use serde::{
    de::{self, Error},
    Deserialize, Serialize,
};
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomerRequest {
    pub api_key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomerIdFromMiddleware(i32);

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct LeakRequest {
    #[serde(deserialize_with = "deserialize_stringified_list")]
    pub supported_identifiers: Vec<Identifier>,
    pub filter: String,
    pub limit: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, TypedBuilder)]
pub struct ResultRequest {
    pub leak_id: String,
    pub received_identities: u32,
    pub number_of_matches: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupportedIdTypes {
    pub email: bool,
    pub phone: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Identifier {
    EMAIL,
    PHONE,
}

// impl DeserializeOwned for Identifier {}

pub fn deserialize_stringified_list<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<Identifier>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringVecVisitor<Identifier>(std::marker::PhantomData<Identifier>);

    impl<'de> de::Visitor<'de> for StringVecVisitor<Identifier> {
        type Value = Vec<Identifier>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a list")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            let mut identifiers: Vec<Identifier> = vec![];
            for value in v.split(',') {
                match value {
                    "EMAIL" => identifiers.push(Identifier::EMAIL),
                    "PHONE" => identifiers.push(Identifier::PHONE),
                    _ => return Err(Error::unknown_variant(v, &["EMAIL", "PHONE"])),
                }
            }
            Ok(identifiers)
        }
    }

    deserializer.deserialize_any(StringVecVisitor(std::marker::PhantomData::<Identifier>))
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EMAIL => write!(f, "EMAIL"),
            Self::PHONE => write!(f, "PHONE"),
        }
    }
}
