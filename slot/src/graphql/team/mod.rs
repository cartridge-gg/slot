use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    response_derives = "Debug",
    schema_path = "schema.json",
    query_path = "src/graphql/team/create.graphql"
)]
pub struct CreateTeam;
#[derive(GraphQLQuery)]
#[graphql(
    response_derives = "Debug",
    schema_path = "schema.json",
    query_path = "src/graphql/team/update.graphql"
)]
pub struct UpdateTeam;

#[derive(GraphQLQuery)]
#[graphql(
    response_derives = "Debug",
    schema_path = "schema.json",
    query_path = "src/graphql/team/members.graphql"
)]
pub struct TeamMembersList;

#[derive(GraphQLQuery)]
#[graphql(
    response_derives = "Debug",
    schema_path = "schema.json",
    query_path = "src/graphql/team/members.graphql"
)]
pub struct TeamMemberAdd;

#[derive(GraphQLQuery)]
#[graphql(
    response_derives = "Debug",
    schema_path = "schema.json",
    query_path = "src/graphql/team/members.graphql"
)]
pub struct TeamMemberRemove;
