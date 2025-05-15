use graphql_client::GraphQLQuery;
use num_bigint::BigInt;

// Query for getting a single paymaster
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug",
    scalars = "BigInt = String"
)]
pub struct GetPaymaster;

// Mutation for creating a paymaster
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug",
    scalars = "BigInt = String"
)]
pub struct CreatePaymaster;

// Mutation for adding policies
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug"
)]
pub struct AddPaymasterPolicies;

// Mutation for removing policies
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug"
)]
pub struct RemovePaymasterPolicies;

// Mutation for increasing budget
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug",
    scalars = "BigInt = String"
)]
pub struct IncreaseBudget;

// Mutation for decreasing budget
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug",
    scalars = "BigInt = String"
)]
pub struct DecreaseBudget;

// Query for listing paymasters
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug",
    scalars = "BigInt = String"
)]
pub struct ListPaymasters;
