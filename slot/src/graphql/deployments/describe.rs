use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    response_derives = "Debug",
    schema_path = "schema.json",
    query_path = "src/graphql/deployments/describe.graphql"
)]
pub struct DescribeDeployment;
