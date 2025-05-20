use graphql_client::GraphQLQuery;

// Query for getting a single paymaster
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug"
)]
pub struct GetPaymaster;

// Mutation for creating a paymaster
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug, Clone"
)]
pub struct CreatePaymaster;

// Mutation for adding policies
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug"
)]
pub struct AddPolicies;

// Mutation for removing policies
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug"
)]
pub struct RemovePolicies;

// Mutation for removing all policies
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug"
)]
pub struct RemoveAllPolicies;

// Mutation for increasing budget
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug, Clone"
)]
pub struct IncreaseBudget;

// Mutation for decreasing budget
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    variables_derives = "Debug, Clone"
)]
pub struct DecreaseBudget;

// Query for listing paymasters
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/paymaster.graphql",
    response_derives = "Debug, Serialize, Clone",
    variables_derives = "Debug"
)]
pub struct ListPaymasters;
