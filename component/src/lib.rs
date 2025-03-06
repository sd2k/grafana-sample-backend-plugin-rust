/// This is an example of a component-based plugin.
///
/// The same traits must be implemented as for a 'v2' gRPC based plugin,
/// but the SDK does something different with the trait implementations.
/// For now the plugin struct must also implement `GrafanaPlugin` manually
/// (there is no derive macro yet), and the `export` macro from the SDK
/// must be used to export the plugin struct manually. This could easily
/// be handled by a derive macro in the SDK in the future, making the API
/// almost identical to the 'v2' gRPC based plugin.
use chrono::prelude::*;
use futures_util::stream;
use grafana_plugin_sdk::{
    backend::{
        export, BoxDataResponseStream, DataQueryError, DataResponse, DataService, DataSourcePlugin,
        Error, GrafanaPlugin, PluginContextT, QueryDataRequest,
    },
    prelude::*,
};
use serde_json::Value;

struct MyPlugin;

impl GrafanaPlugin for MyPlugin {
    /// Indicate that this plugin is a data source plugin with these
    /// JSON and secure JSON types.
    type PluginType = DataSourcePlugin<Self::JsonData, Self::SecureJsonData>;
    type JsonData = Value;
    type SecureJsonData = Value;

    /// Create a new instance of the plugin for this plugin context.
    fn new(_pc: PluginContextT<Self>) -> Result<Self, Error> {
        Ok(MyPlugin)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Error querying backend for {}", .ref_id)]
struct QueryError {
    ref_id: String,
}

impl DataQueryError for QueryError {
    fn ref_id(self) -> String {
        self.ref_id
    }
}

#[async_trait::async_trait]
impl DataService for MyPlugin {
    /// The type of the JSON query sent from Grafana to the plugin.
    type Query = Value;

    /// The error type that can be returned by individual queries.
    ///
    /// This must implement [`DataQueryError`], which allows the SDK to
    /// align queries up with any failed requests.
    type QueryError = QueryError;

    /// The type of stream returned by the `query_data` method.
    /// ]
    /// This will generally be impossible to name directly, so returning the
    /// [`BoxDataResponseStream`] type alias will probably be more convenient.
    type Stream = BoxDataResponseStream<Self::QueryError>;

    /// Query data for an input request.
    ///
    /// The request will contain zero or more queries, as well as information about the
    /// origin of the queries (such as the datasource instance) in the `plugin_context` field.
    async fn query_data(&self, _request: QueryDataRequest<Self::Query, Self>) -> Self::Stream {
        Box::pin(stream::once(async {
            let frame = [
                // Fields can be created from iterators of a variety of
                // relevant datatypes.
                [
                    Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).single().unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 1).single().unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 2).single().unwrap(),
                ]
                .into_field("time"),
                [1_u32, 2, 3].into_field("x"),
                ["a", "b", "c"].into_field("y"),
            ]
            .into_frame("foo");
            Ok(DataResponse::new(
                "A".to_string(),
                vec![frame.check().map_err(|_| QueryError {
                    ref_id: "A".to_string(),
                })?],
            ))
        }))
    }
}

export!(MyPlugin with_types_in grafana_plugin_sdk::pluginv3);
