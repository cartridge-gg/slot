use graphql_client::GraphQLQuery;

// Import Time type from another module (following pattern from other modules)
pub use crate::graphql::paymaster::Time;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/merkle_drop/create.graphql",
    response_derives = "Debug, Clone"
)]
pub struct CreateMerkleDrop;
