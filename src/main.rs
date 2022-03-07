mod data_sources;
mod graphql;

use std::env;

use ::poem::{
    get, handler, middleware::Cors, post, web::Redirect, EndpointExt, IntoResponse, Route,
};
use async_graphql::{dataloader::DataLoader, EmptySubscription, Response, Schema};
use async_graphql_poem::GraphQLRequest;
use aws_sdk_dynamodb::Endpoint;
use data_sources::{
    auth::{Authenticator, Authorizer},
    dishes::DishLoader,
    ingredients::IngredientLoader,
    shares::ShareLoader,
};
use graphql::{Mutation, Query};
use http::Uri;
use poem::web::{
    headers::{authorization::Bearer, Authorization},
    Data, Json, TypedHeader,
};
use poem_lambda::Error;

const DASH_BOARD_URL: &str =
    "https://studio.apollographql.com/sandbox/explorer?endpoint=http://localhost:3000/graphql";

#[derive(Clone)]
enum Profile {
    Prod,
    Local,
}

#[handler]
fn dashboard(Data(profile): Data<&Profile>) -> poem::Response {
    match profile {
        Profile::Local => Redirect::temporary(DASH_BOARD_URL).into_response(),
        Profile::Prod => "ok".into_response(),
    }
}

type TodaysMenuSchema = Schema<Query, Mutation, EmptySubscription>;

#[handler]
async fn index(
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    req: GraphQLRequest,
    schema: Data<&TodaysMenuSchema>,
) -> Json<Response> {
    match auth_header {
        Some(TypedHeader(auth)) => Json(schema.execute(req.0.data(auth)).await),
        _ => Json(schema.execute(req.0).await),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let profile = match env::var("AWS_PROFILE") {
        Ok(p) => {
            if p == "prod" {
                Profile::Prod
            } else {
                Profile::Local
            }
        }
        _ => Profile::Local,
    };
    let config = aws_config::load_from_env().await;
    let dynamodb_local_config = get_dynamodb_config(&profile, &config);
    let s3_local_config = get_s3_config(&profile, config);
    let dynamodb = aws_sdk_dynamodb::Client::from_conf(dynamodb_local_config);
    let s3 = aws_sdk_s3::Client::from_conf(s3_local_config);
    let dish_loader = DishLoader::new(dynamodb.clone(), s3);
    let ingredient_loader = IngredientLoader::new(dynamodb.clone());
    let root_key_hex = env::var("AUTH_PRIVATE_KEY")?;
    let authenticator = Authenticator::new(dynamodb.clone(), &root_key_hex);
    let authorizer = Authorizer::new(&root_key_hex);
    let share_loader = ShareLoader::new(dynamodb);
    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(DataLoader::new(dish_loader, tokio::spawn))
        .data(DataLoader::new(ingredient_loader, tokio::spawn))
        .data(DataLoader::new(share_loader, tokio::spawn))
        .data(authenticator)
        .data(authorizer)
        .finish();
    let cors = Cors::new().allow_origin(env::var("CORS_ORIGIN")?);
    let app = Route::new()
        .at("/", get(dashboard.data(profile)))
        .at("/graphql", post(index.data(schema)).with(cors));
    poem_lambda::run(app).await
}

fn get_s3_config(profile: &Profile, config: aws_config::Config) -> aws_sdk_s3::Config {
    match *profile {
        Profile::Local => aws_sdk_s3::config::Builder::from(&config)
            .endpoint_resolver(Endpoint::immutable(Uri::from_static("http://s3:9000")))
            .build(),
        Profile::Prod => aws_sdk_s3::config::Builder::from(&config).build(),
    }
}

fn get_dynamodb_config(profile: &Profile, config: &aws_config::Config) -> aws_sdk_dynamodb::Config {
    match *profile {
        Profile::Local => aws_sdk_dynamodb::config::Builder::from(config)
            .endpoint_resolver(Endpoint::immutable(Uri::from_static(
                "http://dynamodb:8000",
            )))
            .build(),
        Profile::Prod => aws_sdk_dynamodb::config::Builder::from(config).build(),
    }
}
