use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use async_graphql::dataloader::Loader;
use aws_sdk_dynamodb::{
    model::{AttributeValue, KeysAndAttributes},
    Client,
};
use nanoid::nanoid;
use serde_dynamo::{from_items, to_item};

use crate::graphql::{ShareableMenu, ShareableMenuInput};

const TABLE_NAME: &str = "todays-menu-shares";

pub(crate) struct ShareLoader {
    db_client: Client,
}

#[async_trait::async_trait]
impl Loader<String> for ShareLoader {
    type Value = ShareableMenu;
    type Error = Arc<Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, ShareableMenu>, Self::Error> {
        let keys = KeysAndAttributes::builder()
            .set_keys(Some(
                keys.iter()
                    .map(|k| {
                        let mut map = HashMap::new();
                        map.insert("key".to_string(), AttributeValue::S(k.to_owned()));
                        map
                    })
                    .collect(),
            ))
            .build();
        let output = self
            .db_client
            .batch_get_item()
            .request_items(TABLE_NAME, keys)
            .send()
            .await
            .map_err(|e| Arc::new(e.into()))?;
        match output.responses {
            Some(mut items) => {
                let items = items.remove(TABLE_NAME).expect("responses");
                let dishes: Vec<ShareableMenu> =
                    from_items(items).map_err(|e| Arc::new(e.into()))?;
                Ok(dishes
                    .into_iter()
                    .map(|item| (item.key.clone(), item))
                    .collect())
            }
            None => Ok(HashMap::default()),
        }

        // TODO process unprocessed keys
    }
}

impl ShareLoader {
    pub(crate) fn new(dynamodb: Client) -> ShareLoader {
        ShareLoader {
            db_client: dynamodb,
        }
    }
    pub(crate) async fn create_one(&self, item: ShareableMenuInput) -> Result<String, Error> {
        let key = nanoid!(10);
        self.db_client
            .put_item()
            .table_name(TABLE_NAME)
            .set_item(Some(to_item(item)?))
            .item("key", AttributeValue::S(key.clone()))
            .send()
            .await?;
        Ok(key)
    }
}
