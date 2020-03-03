use graphql_client::GraphQLQuery;
use reqwest::{blocking as req, StatusCode};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github.graphql",
    query_path = "user_query.graphql",
    response_derives = "Debug"
)]
struct UserQuery;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Request failed")]
    RequestError {
        #[from]
        source: reqwest::Error,
    },
    #[error("User unauthorized")]
    Unauthorized,
    #[error("Unsuccessful request with status code: {0}")]
    Unsuccessful(StatusCode),
}

pub fn is_valid_token(token: &str) -> Result<(), Error> {
    let client = req::Client::new();
    let q = UserQuery::build_query(user_query::Variables {});

    let res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .json(&q)
        .send()?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Error::Unauthorized);
    }

    if !response_status.is_success() {
        return Err(Error::Unsuccessful(response_status));
    }

    Ok(())
}
