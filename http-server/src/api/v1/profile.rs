use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::api::error::Error;
use crate::api::Context;

pub fn router(ctx: Context) -> Router {
    Router::new()
        .route("/profile", get(profile_handler))
        .with_state(ctx)
}

/// Return local profile information.
/// `GET /profile`
async fn profile_handler(
    State(ctx): State<Context>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    if !addr.ip().is_loopback() {
        return Err(Error::Auth("Profile data is only shown for localhost"));
    }

    Ok::<_, Error>(Json(
        json!({ "config": ctx.profile.config, "home": ctx.profile.home.path() }),
    ))
}

#[cfg(test)]
mod routes {
    use std::net::SocketAddr;

    use axum::extract::connect_info::MockConnectInfo;
    use axum::http::StatusCode;
    use serde_json::json;

    use crate::test::{self, get};

    #[tokio::test]
    async fn test_remote_profile() {
        let tmp = tempfile::tempdir().unwrap();
        let seed = test::seed(tmp.path());
        let app = super::router(seed.clone())
            .layer(MockConnectInfo(SocketAddr::from(([192, 168, 1, 1], 8080))));
        let response = get(&app, "/profile").await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.json().await,
            json!({
              "error": "Profile data is only shown for localhost",
              "code": 401
            })
        )
    }

    #[tokio::test]
    async fn test_profile() {
        let tmp = tempfile::tempdir().unwrap();
        let seed = test::seed(tmp.path());
        let app = super::router(seed.clone())
            .layer(MockConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))));
        let response = get(&app, "/profile").await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.json().await,
            json!({
              "config": {
                "publicExplorer": "https://app.radicle.xyz/nodes/$host/$rid$path",
                "preferredSeeds": [
                  "z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7@seed.radicle.garden:8776",
                  "z6Mkmqogy2qEM2ummccUthFEaaHvyYmYBYh3dbe9W4ebScxo@ash.radicle.garden:8776"
                ],
                "web": { "pinned": { "repositories": [] } },
                "cli": {
                  "hints": true
                },
                "node": {
                  "alias": "seed",
                  "listen": [],
                  "peers": { "type": "dynamic" },
                  "connect": [],
                  "externalAddresses": [],
                  "network": "main",
                  "log": "INFO",
                  "relay": "auto",
                  "limits": {
                    "routingMaxSize": 1000,
                    "routingMaxAge": 604800,
                    "gossipMaxAge": 1209600,
                    "fetchConcurrency": 1,
                    "maxOpenFiles": 4096,
                    "rate": {
                      "inbound": {
                        "fillRate": 5.0,
                        "capacity": 1024
                      },
                      "outbound": {
                        "fillRate": 10.0,
                        "capacity": 2048
                      }
                    },
                    "connection": {
                      "inbound": 128,
                      "outbound": 16
                    }
                  },
                  "workers": 8,
                  "seedingPolicy": {
                      "default": "block",
                  }
                }
              },
              "home": seed.profile.path()
            })
        );
    }
}
