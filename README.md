# chatterverse_hasura_graphql_client

A graphql client that is designed to make calls to the hasura graphql engine.

## usage:

```toml
[dependencies]
chatterverse_hasura_graphql_client = {git = "https://github.com/chatterverse-ai/chatterverse_hasura_graphql_client.git}
```

then:

```rust
use chatterverse_hasura_graphql_client::HasuraGraphQLClient;

let gql_client = HasuraGraphQLClient::new("https://myapi.com/v1/grapqhl", "my_hasura_admin_secret");
let query = r#"
		query($id: uuid) {
			my_query(where: {id: {_eq: $id}}) {
				id
			}
		}
"#
// can be anything that implements Serialize
let variables = serde_json::json!({
	"id": "25B75811-6866-4081-AF69-2BD27756D66C"
});

let _result: serde_json::Value = gql_client.post_query(
	query,
	variables,
	// if no bearer_token is provided, the queries will be ran as admin via the x-hasura-admin-secret
	None
);
```