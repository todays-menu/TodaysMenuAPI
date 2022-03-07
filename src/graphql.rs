use async_graphql::{
    dataloader::DataLoader, ComplexObject, Context, Enum, Error, InputObject, Object, SimpleObject,
    Upload,
};
use poem::web::headers::{authorization::Bearer, Authorization};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::data_sources::{
    auth::{Authenticator, Authorizer},
    dishes::DishLoader,
    ingredients::IngredientLoader,
    shares::ShareLoader,
};

pub(crate) struct Query;
pub(crate) struct Mutation;

#[Object]
impl Query {
    async fn dishes(&self, ctx: &Context<'_>) -> Result<Vec<Dish>, Error> {
        let loader = ctx.data_unchecked::<DataLoader<DishLoader>>();
        Ok(loader.loader().load_all().await?)
    }
    async fn ingredients(&self, ctx: &Context<'_>) -> Result<Vec<Ingredient>, Error> {
        let loader = ctx.data_unchecked::<DataLoader<IngredientLoader>>();
        Ok(loader.loader().load_all().await?)
    }
    async fn shareable_menu(&self, ctx: &Context<'_>, key: String) -> Result<ShareableMenu, Error> {
        let loader = ctx.data_unchecked::<DataLoader<ShareLoader>>();
        let menu = loader.load_one(key).await?;
        menu.ok_or_else(|| "cannot find shared menu".into())
    }
}

#[Object]
impl Mutation {
    async fn share_menu(
        &self,
        ctx: &Context<'_>,
        menu: ShareableMenuInput,
    ) -> Result<MutationResultWithKey, Error> {
        let auth_header = ctx.data::<Authorization<Bearer>>()?;
        let auth = ctx.data_unchecked::<Authorizer>();
        auth.authorize_share(auth_header.token())?;

        let loader = ctx.data_unchecked::<DataLoader<ShareLoader>>();
        let key = loader.loader().create_one(menu).await?;
        Ok(MutationResultWithKey {
            success: true,
            message: Some("create one shared menu".to_string()),
            key: Some(key),
        })
    }
    async fn add_new_dishes(
        &self,
        ctx: &Context<'_>,
        dishes: Vec<DishInput>,
    ) -> Result<MutationResult, Error> {
        let auth_header = ctx.data::<Authorization<Bearer>>()?;
        let auth = ctx.data_unchecked::<Authorizer>();
        auth.authorize_mutate(auth_header.token())?;

        let loader = ctx.data_unchecked::<DataLoader<DishLoader>>();
        loader.loader().update_many(ctx, &dishes).await?;
        Ok(MutationResult {
            success: true,
            message: Some(format!("added {} dishes", dishes.len())),
        })
    }
    async fn update_dishes(
        &self,
        ctx: &Context<'_>,
        dishes: Vec<DishInput>,
    ) -> Result<MutationResult, Error> {
        let auth_header = ctx.data::<Authorization<Bearer>>()?;
        let auth = ctx.data_unchecked::<Authorizer>();
        auth.authorize_mutate(auth_header.token())?;

        let loader = ctx.data_unchecked::<DataLoader<DishLoader>>();
        loader.loader().update_many(ctx, &dishes).await?;
        Ok(MutationResult {
            success: true,
            message: Some(format!("updated {} dishes", dishes.len())),
        })
    }
    async fn add_new_ingredients(
        &self,
        ctx: &Context<'_>,
        ingredients: Vec<IngredientInput>,
    ) -> Result<MutationResult, Error> {
        let auth_header = ctx.data::<Authorization<Bearer>>()?;
        let auth = ctx.data_unchecked::<Authorizer>();
        auth.authorize_mutate(auth_header.token())?;

        let loader = ctx.data_unchecked::<DataLoader<IngredientLoader>>();
        loader.loader().update_many(&ingredients).await?;
        Ok(MutationResult {
            success: true,
            message: Some(format!("added {} ingredients", ingredients.len())),
        })
    }
    async fn login_user(
        &self,
        ctx: &Context<'_>,
        user_id: String,
        password: String,
    ) -> Result<LoginResult, Error> {
        let authenticator = ctx.data_unchecked::<Authenticator>();
        Ok(authenticator.authenticate(user_id, password).await?)
    }
}

#[derive(SimpleObject, Default, Clone, Serialize, Deserialize)]
#[graphql(rename_fields = "snake_case")]
#[graphql(complex)]
pub(crate) struct Dish {
    pub name: String,
    pub meal: Option<Meal>,
    #[graphql(skip)]
    pub ingredients: Vec<RawRecipeIngredient>,
    pub spicy: Option<f32>,
    pub cook_time: Option<u32>,
    pub recipe_link: Option<String>,
    pub serving: Option<u32>,
    pub one_dish: Option<bool>,
    pub soup: Option<bool>,
    pub style: Option<String>,
    pub photo: Option<Photo>,
    pub suppressed: Option<bool>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct RawRecipeIngredient {
    pub name: String,
    pub quantity: String,
}

#[derive(SimpleObject)]
pub(crate) struct RecipeIngredient {
    pub ingredient: Ingredient,
    pub quantity: String,
}

#[ComplexObject]
impl Dish {
    async fn ingredients(&self, ctx: &Context<'_>) -> Result<Vec<RecipeIngredient>, Error> {
        let loader = ctx.data_unchecked::<DataLoader<IngredientLoader>>();
        let ingredients = loader
            .load_many(
                self.ingredients
                    .iter()
                    .map(|item| item.name.to_owned())
                    .collect::<Vec<_>>(),
            )
            .await?;
        Ok(self
            .ingredients
            .iter()
            .filter(|item| ingredients.contains_key(&item.name))
            .map(|item| RecipeIngredient {
                ingredient: ingredients.get(&item.name).unwrap().clone(),
                quantity: item.quantity.clone(),
            })
            .collect())
    }
}

#[derive(SimpleObject, Clone, Serialize, Deserialize)]
pub(crate) struct Ingredient {
    pub name: String,
    pub category: String,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Meal {
    #[graphql(name = "lunch")]
    Lunch,
    #[graphql(name = "dinner")]
    Dinner,
}

#[derive(SimpleObject, Clone, Serialize, Deserialize)]
pub(crate) struct Photo {
    pub filename: Option<String>,
    pub mimetype: Option<String>,
    pub encoding: Option<String>,
}

#[derive(SimpleObject)]
pub(crate) struct MutationResult {
    success: bool,
    message: Option<String>,
}

#[derive(SimpleObject)]
pub(crate) struct MutationResultWithKey {
    success: bool,
    message: Option<String>,
    // Short shareable key
    key: Option<String>,
}

#[skip_serializing_none]
#[derive(InputObject, Serialize)]
#[graphql(rename_fields = "snake_case")]
pub(crate) struct DishInput {
    pub name: String,
    pub meal: Option<Meal>,
    pub ingredients: Vec<RecipeIngredientInput>,
    pub spicy: Option<f32>,
    pub cook_time: Option<u32>,
    pub recipe_link: Option<String>,
    pub serving: Option<u32>,
    pub one_dish: Option<bool>,
    pub soup: Option<bool>,
    pub style: Option<String>,
    #[serde(skip)]
    pub photo: Option<Upload>,
    pub suppressed: Option<bool>,
}

#[derive(InputObject, Serialize)]
pub(crate) struct RecipeIngredientInput {
    name: String,
    quantity: String,
}

#[derive(InputObject, Serialize)]
pub(crate) struct IngredientInput {
    pub name: String,
    pub category: String,
}

#[derive(SimpleObject, Clone, Deserialize)]
pub(crate) struct ShareableMenu {
    pub key: String,
    /// Opaque base64 encoded payload
    pub payload: String,
}

#[derive(InputObject, Serialize)]
pub(crate) struct ShareableMenuInput {
    /// Opaque base64 encoded payload
    payload: String,
}

#[derive(SimpleObject)]
pub(crate) struct LoginResult {
    pub success: bool,
    pub message: Option<String>,
    pub token: Option<String>,
}
