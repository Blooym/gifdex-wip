mod database;
mod routes;

use crate::routes::{
    handle_index,
    well_known::handle_well_known_did,
    xrpc::{
        com_atproto::sync::handle_get_repo_status,
        health::handle_health,
        net_gifdex::{
            actor::{handle_get_profile, handle_get_profiles},
            feed::{handle_get_post, handle_get_posts_by_actor, handle_get_posts_by_query},
        },
    },
};
use anyhow::{Context, Result};
use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, Method, header},
    middleware::{self as axum_middleware, Next},
    routing::get,
};
use clap::Parser;
use database::Database;
use dotenvy::dotenv;
use gifdex_lexicons::net_gifdex::{
    actor::{get_profile::GetProfileRequest, get_profiles::GetProfilesRequest},
    feed::{
        get_post::GetPostRequest, get_posts_by_actor::GetPostsByActorRequest,
        get_posts_by_query::GetPostsByQueryRequest,
    },
};
use jacquard_api::com_atproto::sync::get_repo_status::GetRepoStatusRequest;
use jacquard_axum::{
    IntoRouter,
    service_auth::{ServiceAuth, ServiceAuthConfig},
};
use jacquard_common::{
    Data, IntoStatic,
    types::{
        did::Did,
        did_doc::{self, DidDocument, Service},
        string::AtprotoStr,
    },
    url::Url,
};
use jacquard_identity::{JacquardResolver, resolver::ResolverOptions};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, signal};
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
    normalize_path::NormalizePathLayer,
    trace::{self, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version)]
struct Arguments {
    /// Local socket address to serve the AppView on.
    #[arg(
        long = "address",
        env = "GIFDEX_APPVIEW_ADDRESS",
        default_value = "127.0.0.1:8255"
    )]
    address: SocketAddr,

    /// Postgres database to use for AppView data storage.
    ///
    /// This should be the same database used for all other services that read/write application data.
    #[arg(
        long = "database-url",
        env = "GIFDEX_APPVIEW_DATABASE_URL",
        env = "DATABASE_URL"
    )]
    database_url: String,

    /// Host to use for serving media from.
    ///
    /// The host must serve Gifdex-compatiable endpoints with the expected formats.
    #[arg(long = "cdn", env = "GIFDEX_APPVIEW_CDN")]
    cdn: Url,

    /// Host that this AppView will reachable by.
    ///
    /// Used for generating a `well-known/did.json` document, `did:web` identity and a AppView service endpoint.
    #[arg(long = "host", env = "GIFDEX_APPVIEW_HOST")]
    host: Url,
}

#[derive(Clone)]
struct AppState {
    database: Arc<Database>,
    cdn_url: Url,
    service_did_document: DidDocument<'static>,
    service_auth_config: ServiceAuthConfig<JacquardResolver>,
}

impl ServiceAuth for AppState {
    type Resolver = JacquardResolver;

    fn service_did(&self) -> &Did<'_> {
        self.service_auth_config.service_did()
    }

    fn resolver(&self) -> &Self::Resolver {
        self.service_auth_config.resolver()
    }

    fn require_lxm(&self) -> bool {
        ServiceAuth::require_lxm(&self.service_auth_config)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .init();
    let args = Arguments::parse();

    // Create ATProto service information.
    let service_did = Did::new_owned(format!(
        "did:web:{}",
        args.host
            .host_str()
            .context("unable to get host from host url")?
    ))
    .context("failed to create did:web from host")?;
    let service_did_doc = build_service_did_doc(&service_did, &args.host);
    let service_auth_config = ServiceAuthConfig::new(
        service_did,
        JacquardResolver::new(reqwest::Client::new(), ResolverOptions::default()),
    );

    // Initialise application state and required services.
    let database = Arc::new(
        Database::new(&args.database_url)
            .await
            .context("failed to connect to database")?,
    );

    // Start server.
    let router = Router::new()
        .route("/", get(handle_index))
        .route("/xrpc/_health", get(handle_health))
        .route("/.well-known/did.json", get(handle_well_known_did))
        // AtProto Sync
        .merge(GetRepoStatusRequest::into_router(handle_get_repo_status))
        // Gifdex Actor
        .merge(GetProfileRequest::into_router(handle_get_profile))
        .merge(GetProfilesRequest::into_router(handle_get_profiles))
        // Gifdex Feed
        .merge(GetPostRequest::into_router(handle_get_post))
        .merge(GetPostsByQueryRequest::into_router(
            handle_get_posts_by_query,
        ))
        .merge(GetPostsByActorRequest::into_router(
            handle_get_posts_by_actor,
        ))
        // Gifdex Moderation
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::default().level(Level::INFO))
                .on_response(DefaultOnResponse::default().level(Level::INFO))
                .on_failure(DefaultOnFailure::default().level(Level::ERROR)),
        )
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(CatchPanicLayer::new()) // TODO: Use custom panic handler to return Xrpc InternalServerError.
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any),
        )
        .layer(axum_middleware::from_fn(
            async |req: Request, next: Next| {
                let mut res = next.run(req).await;
                let res_headers = res.headers_mut();
                res_headers.insert(
                    header::SERVER,
                    HeaderValue::from_static(env!("CARGO_PKG_NAME")),
                );
                res_headers.insert("X-Robots-Tag", HeaderValue::from_static("none"));
                res
            },
        ))
        .with_state(AppState {
            database,
            cdn_url: args.cdn,
            service_did_document: service_did_doc,
            service_auth_config,
        });

    let tcp_listener = TcpListener::bind(args.address).await?;
    info!(
        "Internal server started - listening on: http://{}",
        args.address,
    );
    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn build_service_did_doc(did: &Did<'_>, url: &Url) -> DidDocument<'static> {
    DidDocument::new()
        .context(did_doc::default_context())
        .id(did.clone())
        .service(vec![
            Service::new()
                .id("#gifdex_appview".into())
                .r#type("GifdexAppView".into())
                .service_endpoint(Data::String(AtprotoStr::String(
                    url.as_str().trim_end_matches("/").into(),
                )))
                .extra_data(BTreeMap::default())
                .build(),
        ])
        .extra_data(BTreeMap::default())
        .build()
        .into_static()
}
