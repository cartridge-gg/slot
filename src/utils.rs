pub fn format_graphql_errors(errors: &[graphql_client::Error]) -> String {
    errors
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<&str>>()
        .join("\n")
}
