wit_bindgen::generate!({
    world: "backend",
    path: "wit",
    generate_all,
    async: true,
});

use exports::grafana::plugins::query_data::{Error, Guest};
use grafana::plugins::types::{DataResponse, QueryDataRequest, QueryDataResponse};

struct MyPlugin;

impl Guest for MyPlugin {
    async fn query_data(request: QueryDataRequest) -> Result<QueryDataResponse, Error> {
        Ok(QueryDataResponse {
            responses: vec![("foo".to_string(), DataResponse { frames: vec![] })],
        })
    }
}

export!(MyPlugin);
