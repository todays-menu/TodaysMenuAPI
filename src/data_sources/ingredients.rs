use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use async_graphql::dataloader::Loader;
use aws_sdk_dynamodb::{
    model::{AttributeValue, KeysAndAttributes},
    Client,
};
use serde_dynamo::from_items;

use crate::graphql::{Ingredient, IngredientInput};

const TABLE_NAME: &str = "todays-menu-ingredients";

const UPDATE_EXP: &str = "SET
    #C = :category
";

pub(crate) struct IngredientLoader {
    db_client: Client,
}

#[async_trait::async_trait]
impl Loader<String> for IngredientLoader {
    type Value = Ingredient;
    type Error = Arc<Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Ingredient>, Self::Error> {
        let mut ingredients: Vec<Ingredient> = vec![];
        for keys in keys.chunks(100) {
            let keys = KeysAndAttributes::builder()
                .set_keys(Some(
                    keys.iter()
                        .map(|k| {
                            let mut map = HashMap::new();
                            map.insert("name".to_string(), AttributeValue::S(k.to_owned()));
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
            if let Some(mut items) = output.responses {
                let items = items.remove(TABLE_NAME).expect("responses");
                ingredients.append(&mut from_items(items).map_err(|e| Arc::new(e.into()))?);
            }
        }
        Ok(ingredients
            .into_iter()
            .map(|item| (item.name.clone(), item))
            .collect())
        // TODO process unprocessed keys
    }
}

impl IngredientLoader {
    pub(crate) fn new(dynamodb: Client) -> IngredientLoader {
        IngredientLoader {
            db_client: dynamodb,
        }
    }
    pub(crate) async fn load_all(&self) -> Result<Vec<Ingredient>, Error> {
        let output = self.db_client.scan().table_name(TABLE_NAME).send().await?;
        match output.items {
            Some(items) => Ok(from_items(items)?),
            None => Ok(vec![]),
        }
    }
    pub(crate) async fn update_many(&self, items: &[IngredientInput]) -> Result<(), Error> {
        for item in items {
            self.db_client
                .update_item()
                .table_name(TABLE_NAME)
                .key("name", AttributeValue::S(item.name.clone()))
                .update_expression(UPDATE_EXP)
                .expression_attribute_names("#C", "category")
                .expression_attribute_values(":category", AttributeValue::S(item.category.clone()))
                .send()
                .await?;
        }

        // TODO handle error
        Ok(())
    }
}
