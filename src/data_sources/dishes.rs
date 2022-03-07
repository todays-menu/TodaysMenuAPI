use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use async_graphql::{dataloader::Loader, Context};
use aws_sdk_dynamodb::{
    model::{AttributeValue, KeysAndAttributes},
    Client as DynamoDbClient,
};
use aws_sdk_s3::{types::ByteStream, Client as S3Client};
use nanoid::nanoid;
use serde_dynamo::{from_items, to_attribute_value};
use tokio::fs::File;

use crate::graphql::{Dish, DishInput, Photo};

const TABLE_NAME: &str = "todays-menu-dishes";
const S3_BUCKET: &str = "todays-menu-photos";

const UPDATE_EXP: &str = "SET
    meal = :meal,
    ingredients = :ingredients,
    spicy = :spicy,
    cook_time = :cook_time,
    recipe_link = :recipe_link,
    serving = :serving,
    one_dish = :one_dish,
    soup = :soup,
    #st = :style,
    suppressed = :suppressed
";

const UPDATE_EXP_WITH_PHOTO: &str = "SET
    meal = :meal,
    ingredients = :ingredients,
    spicy = :spicy,
    cook_time = :cook_time,
    recipe_link = :recipe_link,
    serving = :serving,
    one_dish = :one_dish,
    soup = :soup,
    #st = :style,
    suppressed = :suppressed,
    photo = :photo
";

pub(crate) struct DishLoader {
    db_client: DynamoDbClient,
    s3_client: S3Client,
}

#[async_trait::async_trait]
impl Loader<String> for DishLoader {
    type Value = Dish;
    type Error = Arc<Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Dish>, Self::Error> {
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
        match output.responses {
            Some(mut items) => {
                let items = items.remove(TABLE_NAME).expect("responses");
                let dishes: Vec<Dish> = from_items(items).map_err(|e| Arc::new(e.into()))?;
                Ok(dishes
                    .into_iter()
                    .map(|item| (item.name.clone(), item))
                    .collect())
            }
            None => Ok(HashMap::default()),
        }

        // TODO process unprocessed keys
    }
}

impl DishLoader {
    pub(crate) fn new(dynamodb: DynamoDbClient, s3: S3Client) -> DishLoader {
        DishLoader {
            db_client: dynamodb,
            s3_client: s3,
        }
    }
    pub(crate) async fn load_all(&self) -> Result<Vec<Dish>, Error> {
        let output = self.db_client.scan().table_name(TABLE_NAME).send().await?;
        match output.items {
            Some(items) => Ok(from_items(items)?),
            None => Ok(vec![]),
        }
    }
    pub(crate) async fn update_many(
        &self,
        ctx: &Context<'_>,
        items: &[DishInput],
    ) -> Result<(), Error> {
        for item in items {
            let mut update_item = self
                .db_client
                .update_item()
                .table_name(TABLE_NAME)
                .key("name", AttributeValue::S(item.name.clone()))
                .update_expression(UPDATE_EXP);
            if let Some(photo) = &item.photo {
                let upload_value = photo.value(ctx)?;
                let key = format!("{}.jpg", nanoid!());
                self.s3_client
                    .put_object()
                    .bucket(S3_BUCKET)
                    .key(key.clone())
                    .content_type(
                        upload_value
                            .content_type
                            .unwrap_or_else(|| "image/jpeg".to_string()),
                    )
                    .body(ByteStream::from_file(File::from_std(upload_value.content)).await?)
                    .send()
                    .await?;
                update_item = update_item
                    .set_update_expression(Some(UPDATE_EXP_WITH_PHOTO.to_string()))
                    .expression_attribute_values(
                        ":photo",
                        to_attribute_value(Photo {
                            filename: Some(key),
                            mimetype: None,
                            encoding: None,
                        })?,
                    );
            }
            update_item
                .expression_attribute_names("#st", "style")
                .expression_attribute_values(":meal", to_attribute_value(&item.meal)?)
                .expression_attribute_values(":ingredients", to_attribute_value(&item.ingredients)?)
                .expression_attribute_values(":spicy", to_attribute_value(&item.spicy)?)
                .expression_attribute_values(":cook_time", to_attribute_value(&item.cook_time)?)
                .expression_attribute_values(":recipe_link", to_attribute_value(&item.recipe_link)?)
                .expression_attribute_values(":serving", to_attribute_value(&item.serving)?)
                .expression_attribute_values(":one_dish", to_attribute_value(&item.one_dish)?)
                .expression_attribute_values(":soup", to_attribute_value(&item.soup)?)
                .expression_attribute_values(":style", to_attribute_value(&item.style)?)
                .expression_attribute_values(":suppressed", to_attribute_value(&item.suppressed)?)
                .send()
                .await?;
        }
        // TODO handle error
        Ok(())
    }
}
