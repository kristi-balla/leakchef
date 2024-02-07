use crusty::LiveFeedCrypto;

use crate::{Identifier, MappedIdentity, PartialIdentity, RoutesError};

fn salt_emails_of_identity(
    identity: PartialIdentity,
    salt: &[u8],
    crypto: &LiveFeedCrypto,
) -> Result<Vec<String>, RoutesError> {
    let mut salted_emails: Vec<String> = vec![];
    for email in identity.emails {
        let salted_email = crypto.apply_salt(&email, salt).map_err(|err| {
            tracing::error!("Something went wrong while applying salt to identifier: {err:?}");
            RoutesError::Crypto
        })?;

        salted_emails.push(salted_email);
    }

    Ok(salted_emails)
}

fn salt_phones_of_identity(
    identity: PartialIdentity,
    salt: &[u8],
    crypto: &LiveFeedCrypto,
) -> Result<Vec<String>, RoutesError> {
    let mut salted_phones: Vec<String> = vec![];
    for phone in identity.phones {
        let salted_phone = crypto.apply_salt(&phone, salt).map_err(|err| {
            tracing::error!("Something went wrong while applying salt to identifier: {err:?}");
            RoutesError::Crypto
        })?;

        salted_phones.push(salted_phone);
    }

    Ok(salted_phones)
}

pub fn salt_identifier(
    identities: Vec<PartialIdentity>,
    supported_identifiers: &[Identifier],
    salt: String,
    crypto: &LiveFeedCrypto,
) -> Result<Vec<MappedIdentity>, RoutesError> {
    let mut reply_identities: Vec<MappedIdentity> = vec![];
    for partial_identity in identities {
        let emails: Vec<String> = if supported_identifiers.contains(&Identifier::EMAIL) {
            salt_emails_of_identity(partial_identity.clone(), salt.as_bytes(), crypto)?
        } else {
            vec![]
        };

        let phones: Vec<String> = if supported_identifiers.contains(&Identifier::PHONE) {
            salt_phones_of_identity(partial_identity.clone(), salt.as_bytes(), crypto)?
        } else {
            vec![]
        };

        // here you should construct a parsedIdentity and then a mappedIdentity
        let salted_identity = PartialIdentity::builder()
            .object_id(partial_identity.object_id)
            .domains(partial_identity.domains)
            .passwords(partial_identity.passwords)
            .emails(emails)
            .phones(phones)
            .build();

        let mapped_identity = MappedIdentity::from(salted_identity);

        reply_identities.push(mapped_identity);
    }
    Ok(reply_identities)
}
