pub use crate::graphql::deployments::Time;
use graphql_client::GraphQLQuery;

// Query for listing policies from a paymaster
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster/list_policies.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug"
)]
pub struct ListPolicies;

// Mutation for creating a paymaster
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster/create.graphql",
    variables_derives = "Debug, Clone"
)]
pub struct CreatePaymaster;

// Mutation for adding policies
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster/add_policies.graphql",
    variables_derives = "Debug"
)]
pub struct AddPolicies;

// Mutation for removing policies
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster/remove_policies.graphql",
    variables_derives = "Debug"
)]
pub struct RemovePolicies;

// Mutation for removing all policies
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster/remove_all_policies.graphql",
    variables_derives = "Debug"
)]
pub struct RemoveAllPolicies;

// Mutation for increasing budget
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster/increase_budget.graphql",
    variables_derives = "Debug, Clone"
)]
pub struct IncreaseBudget;

// Mutation for decreasing budget
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster/decrease_budget.graphql",
    variables_derives = "Debug, Clone"
)]
pub struct DecreaseBudget;

// Query for listing paymasters
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster/list_paymasters.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug"
)]
pub struct ListPaymasters;

// Query for paymaster stats
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster/paymaster_stats.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug"
)]
pub struct PaymasterStats;
