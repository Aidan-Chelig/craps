use serde_json::json;
use tokio::{pin, runtime, select};

//imports for tracing
//

use lazy_static::lazy_static;

use prometheus::{Opts, Counter, TextEncoder, Encoder, Gauge, IntGauge, register_int_gauge, register_gauge};
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing::{span, Level, instrument, event, error};

use opentelemetry::{global, KeyValue, sdk::Resource};
use opentelemetry_prometheus::PrometheusExporter;

use tracing::Instrument;
use futures::{sink::SinkExt, stream::StreamExt};


use axum::{
    AddExtensionLayer,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
        Extension,
    },
    handler::get,
    response::IntoResponse,
    Router,
};
use uuid::Uuid;
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use std::sync::{Arc, Mutex};

mod game;

use game::game::{Game, GameState};
use game::craps_messages::*;
use game::user::User;

lazy_static! {
    static ref CONNECTIONS_GAUGE: IntGauge =
        register_int_gauge!("connections", "Number of consecutive connections").unwrap();

    static ref POT_GAUGE: Gauge =
        register_gauge!("pot", "Amount of money in the pot").unwrap();
}

#[instrument]
async fn setup_tracing() -> PrometheusExporter {

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("craps server")
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("err");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    //define subscribers of traces
    let subscriber = Registry::default()
        .with(telemetry)
        .with(tracing_tracy::TracyLayer::new())
        //with filter
        .with(tracing_subscriber::fmt::layer().pretty());

    tracing::subscriber::set_global_default(subscriber).unwrap();

    opentelemetry_prometheus::exporter()
        .with_resource(Resource::new(vec![KeyValue::new("R","V")]))
        .init()


    //TODO prometheus
}

#[tokio::main]
#[instrument]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "craps=info,tower_http=info")
    }

    //let tracer = opentelemetry_jaeger::new_pipeline().install_batch(opentelemetry::runtime::Tokio);


    let exporter = setup_tracing().await;
    let meter = global::meter("craps");

    let counter = meter
        .u64_counter("connections.counter")
        .with_description("connection")
        .init();


    let _root = span!(Level::INFO, "app_start").entered();

    let game = Arc::new(Mutex::new(Game::new()));
    let gamestate = Arc::new(GameState::new(game));



    // build our application with the websocket rout
    let app = Router::new()
        // routes are matched from bottom to top, so we have to put `nest` at the
        // top since it matches all routes
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(AddExtensionLayer::new(gamestate));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


#[instrument]
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    Extension(state): Extension<Arc<GameState>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        event!(Level::INFO, "`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[instrument(skip(socket))]
async fn handle_socket(mut socket: WebSocket, state: Arc<GameState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut uuid: Uuid = Uuid::nil();
    let mut user: User;


    let mut rx = state.tx.clone();
    if let Some(msg) = receiver.next().instrument(tracing::debug_span!("Wait for first message")).await {
        if let Ok(msg) = msg {
            if let Ok(msg) = msg.into_text() {
                if msg.parse::<u16>().unwrap() == CrapsAction::Join as u16 {
                    let mut user_set = state.user_set.lock().unwrap();
                    uuid = Uuid::new_v4();
                    user = User::new(uuid);
                    user_set.insert(uuid, user);
                    event!(Level::DEBUG,"New user added with uuid : {}", uuid);
                }

            }
        } else {
            event!(Level::INFO, "Client Disconnected");
            return;
        }

    }

    if uuid.is_nil() {
        event!(Level::INFO, "Client Disconnect, Incorrect handshake message");
        return;
    }

    let msg = json!({
        "action": "0",
        "message": "",
    });

    if sender.send(Message::Text(msg.to_string())).await.is_err() {
        event!(Level::ERROR, "couldnt send test message, prolly sumn fucked up prolly");
        return;
    };




}
