use graphql_client::GraphQLQuery;
use starknet_types_core::felt::Felt;

// Import Time type from another module (following pattern from other modules)
pub use crate::paymaster::Time;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/merkle_drop/create.graphql",
    response_derives = "Debug, Clone"
)]
pub struct CreateMerkleDrop;
