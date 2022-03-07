use anyhow::Error;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use biscuit_auth::{Biscuit, KeyPair, PrivateKey};
use serde::Deserialize;
use serde_dynamo::from_item;

use crate::graphql::LoginResult;

const TABLE_NAME: &str = "todays-menu-users";

pub(crate) struct Authenticator {
    db_client: Client,
    root_key: KeyPair,
}

#[derive(Deserialize)]
struct UserRecord {
    user_id: String,
    roles: Vec<String>,
    hashed_password: String,
}

impl Authenticator {
    pub(crate) fn new(dynamodb: Client, root_key_hex: &str) -> Authenticator {
        let root_key_bytes = hex::decode(root_key_hex).expect("invalid root key hexstring");
        let root_key = KeyPair::from(
            PrivateKey::from_bytes(&root_key_bytes).expect("invalid root key hexstring"),
        );

        Authenticator {
            db_client: dynamodb,
            root_key,
        }
    }

    pub(crate) async fn authenticate(
        &self,
        user_id: String,
        password: String,
    ) -> Result<LoginResult, Error> {
        let output = self
            .db_client
            .get_item()
            .table_name(TABLE_NAME)
            .key("user_id", AttributeValue::S(user_id.clone()))
            .send()
            .await?;

        let user: UserRecord = match output.item.map(from_item).transpose() {
            Ok(Some(user)) => user,
            _ => {
                let mut builder = Biscuit::builder(&self.root_key);
                builder.add_authority_fact(format!("user(\"{}\")", user_id).as_str())?;
                let token = builder.build()?.to_base64()?;
                return Ok(LoginResult {
                    success: true,
                    message: Some("readonly user".to_string()),
                    token: Some(token),
                });
            }
        };

        let password_hash =
            PasswordHash::new(&user.hashed_password).expect("unable to parse stored password hash");
        Ok(
            match Argon2::default().verify_password(password.as_bytes(), &password_hash) {
                Ok(()) => {
                    let mut builder = Biscuit::builder(&self.root_key);
                    builder.add_authority_fact(format!("user(\"{}\")", user.user_id).as_str())?;
                    for role in user.roles {
                        builder.add_authority_fact(
                            format!("role(\"{}\", \"{}\")", user.user_id, role).as_str(),
                        )?;
                    }
                    let token = builder.build()?.to_base64()?;
                    LoginResult {
                        success: true,
                        message: Some("authenticated".to_string()),
                        token: Some(token),
                    }
                }
                Err(_) => LoginResult {
                    success: false,
                    message: Some("incorrect password".to_string()),
                    token: None,
                },
            },
        )
    }
}

pub(crate) struct Authorizer {
    root_key: KeyPair,
}

impl Authorizer {
    pub(crate) fn new(root_key_hex: &str) -> Authorizer {
        let root_key_bytes = hex::decode(root_key_hex).expect("invalid root key hexstring");
        let root_key = KeyPair::from(
            PrivateKey::from_bytes(&root_key_bytes).expect("invalid root key hexstring"),
        );

        Authorizer { root_key }
    }
    pub(crate) fn authorize_mutate(&self, token: &str) -> Result<(), Error> {
        let biscuit = Biscuit::from_base64(token, |_| self.root_key.public())?;
        let mut authorizer = biscuit.authorizer()?;
        authorizer.add_policy("allow if user($user_id), role($user_id, \"admin\")")?;
        authorizer.add_policy("deny if true")?;
        authorizer.authorize()?;
        Ok(())
    }
    pub(crate) fn authorize_share(&self, token: &str) -> Result<(), Error> {
        let biscuit = Biscuit::from_base64(token, |_| self.root_key.public())?;
        let mut authorizer = biscuit.authorizer()?;
        authorizer.add_policy("allow if user($user_id)")?;
        authorizer.authorize()?;
        Ok(())
    }
}
