use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};

use bytes::Bytes;
use chrono::prelude::*;
use futures::{stream::FuturesOrdered, Stream};
use http::Response;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::oneshot;
use tokio_stream::StreamExt;
use tracing::{debug, info};

use grafana_plugin_sdk::{backend, data, prelude::*};

#[derive(Clone, Debug, Default)]
struct MyPluginService(Arc<AtomicUsize>);

impl MyPluginService {
    fn new() -> Self {
        Self(Arc::new(AtomicUsize::new(0)))
    }
}

#[derive(Debug, Error)]
#[error("Error querying backend for {}", .ref_id)]
struct QueryError {
    ref_id: String,
}

impl backend::DataQueryError for QueryError {
    fn ref_id(self) -> String {
        self.ref_id
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MyQuery {
    path: Option<String>,
    constant: f64,
    #[serde(rename = "withStreaming")]
    with_streaming: bool,
}

#[backend::async_trait]
impl backend::DataService for MyPluginService {
    type Query = MyQuery;
    type QueryError = QueryError;
    type Stream = backend::BoxDataResponseStream<Self::QueryError>;
    async fn query_data(&self, request: backend::QueryDataRequest<Self::Query>) -> Self::Stream {
        let uid = request
            .plugin_context
            .datasource_instance_settings
            .as_ref()
            .map(|ds| ds.uid.clone());

        info!(rows = 3, "Querying data");
        Box::pin(
            request
                .queries
                .into_iter()
                .map(|x| {
                    let uid = uid.clone();
                    async move {
                        // Here we create a single response Frame for each query.
                        // Frames can be created from iterators of fields using [`IntoFrame`].
                        let mut frame = [
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

                        if let Some(uid) = &uid {
                            if let Some("stream") = x.query.path.as_deref() {
                                frame.set_channel(format!("ds/{uid}/stream").parse().unwrap());
                            }
                        }

                        Ok(backend::DataResponse::new(
                            x.ref_id.clone(),
                            vec![frame.check().map_err(|_| QueryError { ref_id: x.ref_id })?],
                        ))
                    }
                })
                .collect::<FuturesOrdered<_>>(),
        )
    }
}

#[derive(Debug, Error)]
#[error("Error streaming data")]
enum StreamError {
    #[error("Error converting frame: {0}")]
    Conversion(#[from] backend::ConvertToError),
    #[error("Invalid frame returned: {0}")]
    InvalidFrame(#[from] data::Error),
}

pub struct ClientDisconnect<T>(T, oneshot::Sender<()>);

impl<T, I> Stream for ClientDisconnect<T>
where
    T: Stream<Item = I> + std::marker::Unpin,
{
    type Item = I;
    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(ctx)
    }
}

#[backend::async_trait]
impl backend::StreamService for MyPluginService {
    type JsonValue = ();
    async fn subscribe_stream(
        &self,
        request: backend::SubscribeStreamRequest,
    ) -> Result<backend::SubscribeStreamResponse, Self::Error> {
        info!(path = %request.path, "Subscribing to stream");
        if request.path.as_str() == "stream" {
            Ok(backend::SubscribeStreamResponse::ok(None))
        } else {
            Ok(backend::SubscribeStreamResponse::not_found())
        }
    }

    type Error = StreamError;
    type Stream = ClientDisconnect<backend::BoxRunStream<Self::Error>>;
    async fn run_stream(
        &self,
        request: backend::RunStreamRequest,
    ) -> Result<Self::Stream, Self::Error> {
        info!(path = %request.path, "Running stream");
        let mut x = 0u32;
        let n = 3;
        let initial_data: [u32; 0] = [];
        let mut frame = data::Frame::new("foo").with_field(initial_data.into_field("x"));

        let stream = Box::pin(
            async_stream::try_stream! {
                loop {
                    frame.fields_mut()[0].set_values(
                        (x..x+n)
                    )?;
                    let packet = backend::StreamPacket::from_frame(frame.check()?)?;
                    debug!("Yielding frame from {} to {}", x, x+n);
                    yield packet;
                    x += n;
                }
            }
            .throttle(Duration::from_secs(1)),
        );

        let (tx, rx) = oneshot::channel();
        let datasource_id = request
            .plugin_context
            .datasource_instance_settings
            .map(|x| x.uid);
        let path = request.path;
        tokio::spawn(async move {
            let _ = rx.await;
            info!(
                "client disconnected for {}path {}",
                datasource_id
                    .as_ref()
                    .map(|x| format!("datasource {x}, "))
                    .unwrap_or_else(|| "".to_string()),
                path
            );
        });
        Ok(ClientDisconnect(stream, tx))
    }

    async fn publish_stream(
        &self,
        _request: backend::PublishStreamRequest,
    ) -> Result<backend::PublishStreamResponse, Self::Error> {
        info!("Publishing to stream");
        todo!()
    }
}

#[derive(Debug, Error)]
enum ResourceError {
    #[error("HTTP error: {0}")]
    Http(#[from] http::Error),

    #[error("Not found")]
    NotFound,
}

impl backend::ErrIntoHttpResponse for ResourceError {
    fn into_http_response(self) -> Result<http::Response<Bytes>, Box<dyn std::error::Error>> {
        let status = match &self {
            Self::Http(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => http::StatusCode::NOT_FOUND,
        };
        Ok(Response::builder()
            .status(status)
            .header(http::header::CONTENT_TYPE, "text/plain")
            .body(Bytes::from(self.to_string()))?)
    }
}

#[backend::async_trait]
impl backend::ResourceService for MyPluginService {
    type Error = ResourceError;
    type InitialResponse = http::Response<Bytes>;
    type Stream = backend::BoxResourceStream<Self::Error>;
    async fn call_resource(
        &self,
        r: backend::CallResourceRequest,
    ) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
        let count = Arc::clone(&self.0);
        let response_and_stream = match r.request.uri().path() {
            // Just send back a single response.
            "/echo" => Ok((
                Response::new(r.request.into_body()),
                Box::pin(futures::stream::empty()) as Self::Stream,
            )),
            // Send an initial response with the current count, then stream the gradually
            // incrementing count back to the client.
            "/count" => Ok((
                Response::new(
                    count
                        .fetch_add(1, Ordering::SeqCst)
                        .to_string()
                        .into_bytes()
                        .into(),
                ),
                Box::pin(async_stream::try_stream! {
                    loop {
                        let body = count
                            .fetch_add(1, Ordering::SeqCst)
                            .to_string()
                            .into_bytes()
                            .into();
                        yield body;
                    }
                }) as Self::Stream,
            )),
            _ => return Err(ResourceError::NotFound),
        };
        response_and_stream
    }
}

#[grafana_plugin_sdk::main(
    services(data, resource, stream),
    init_subscriber = true,
    shutdown_handler = "0.0.0.0:10002"
)]
async fn plugin() -> MyPluginService {
    MyPluginService::new()
}
