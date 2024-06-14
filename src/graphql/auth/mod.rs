use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/auth/info.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct Me;
