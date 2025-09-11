use graphql_client::GraphQLQuery;

pub type Time = String;
pub type Cursor = String;

// Mutation for creating an RPC API key
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/rpc/create_token.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug"
)]
pub struct CreateRpcApiKey;

// Mutation for deleting an RPC API key
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/rpc/delete_token.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug"
)]
pub struct DeleteRpcApiKey;

// Query for listing RPC API keys - commented out for now due to complex schema types
// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "schema.json",
//     query_path = "src/graphql/rpc/list_tokens.graphql",
//     response_derives = "Debug, Serialize, Clone",
//     variables_derives = "Debug"
// )]
// pub struct ListRpcApiKeys;

// Mutation for creating RPC CORS domain
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/rpc/add_whitelist_origin.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug"
)]
pub struct CreateRpcCorsDomain;

// Mutation for deleting RPC CORS domain
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/rpc/remove_whitelist_origin.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug"
)]
pub struct DeleteRpcCorsDomain;

// Query for listing RPC CORS domains - commented out for now due to complex schema types
// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "schema.json",
//     query_path = "src/graphql/rpc/list_whitelist_origins.graphql",
//     response_derives = "Debug, Serialize, Clone",
//     variables_derives = "Debug"
// )]
// pub struct ListRpcCorsDomains;
