use crate::middleware::auth::{check_admin_auth, user_manager};
use domination_web::{web_files, Dir};
use serde::{Deserialize, Serialize};

use crate::{
    app::client::AppClient,
    bt,
    game::GameConfig,
    hardware::wifi::WifiConfig,
    http::{server::HttpServer, ContentType, Json, Response, ResponseBody},
};

pub fn routes(server: &mut HttpServer) {
    #[derive(Debug, Clone, Copy, Deserialize)]
    struct EmptyRequest {}

    server.get("/game/progress", |_, _| {
        let client = AppClient::get();
        let progress = client.get_match_progress()?;
        Ok(Json::new(&progress)?.into())
    });

    server.post("/game/start", |_: EmptyRequest, _, ctx| {
        check_admin_auth(ctx)?;
        let client = AppClient::get();
        client.start_game()?;
        Ok(Response::ok())
    });

    server.post("/game/stop", |_: EmptyRequest, _, ctx| {
        check_admin_auth(ctx)?;
        let client = AppClient::get();
        client.stop_game()?;
        Ok(Response::ok())
    });

    server.post("/game/config", |config: GameConfig, _, ctx| {
        check_admin_auth(ctx)?;
        let client = AppClient::get();
        client.update_game_config(config)?;
        Ok(Response::ok())
    });

    server.get("/game/config", |_, _| {
        let client = AppClient::get();
        let config = client.get_game_config()?;
        Ok(Json::new(&config)?.into())
    });

    #[derive(Debug, Clone, Deserialize)]
    struct ConfigureRequest {
        wifi_config: WifiConfig,
    }

    server.post("/app/config", |body: ConfigureRequest, _, _| {
        let client = AppClient::get();
        client.setup_wifi(body.wifi_config)?;
        Ok(Response::ok())
    });

    #[derive(Debug, Clone, Serialize)]
    struct GetAppConfigRequest {
        wifi_config: Option<WifiConfig>,
    }

    server.get("/app/config", |_, _| {
        let client = AppClient::get();
        let wifi_config = client.get_wifi_config()?;
        Ok(Json::new(&GetAppConfigRequest { wifi_config })?.into())
    });

    server.get("/app/status", |_, _| {
        let client = AppClient::get();
        let app_status = client.get_app_state()?;
        Ok(Json::new(&app_status)?.into())
    });

    #[derive(Debug, Clone, Deserialize)]
    struct AuthChallengeRequest {
        username: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct AuthChallengeResponse {
        nonce: String,
    }

    server.post("/auth/challenge", |body: AuthChallengeRequest, _, _| {
        let user_manager = user_manager();
        let nonce = user_manager
            .generate_nonce(body.username)
            .ok_or_else(|| anyhow::anyhow!("Falha ao gerar nonce"))?;
        Ok(Json::new(&AuthChallengeResponse { nonce })?.into())
    });

    #[derive(Debug, Clone, Deserialize)]
    struct LoginRequest {
        username: String,
        password: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct LoginResponse {
        token: String,
    }

    server.post("/auth/login", |body: LoginRequest, _, ctx| {
        let ip = ctx
            .client_ip
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Falha ao gerar token"))?;
        let user_manager = user_manager();
        let token = user_manager
            .generate_token(body.username, body.password, ip)
            .map_err(|_| anyhow::anyhow!("Falha ao gerar token"))?;
        Ok(Json::new(&LoginResponse {
            token: token.into_string(),
        })?
        .into())
    });

    server.get("/bt/sinks", |_, ctx| {
        check_admin_auth(ctx)?;
        Ok(Json::new(&bt::list_sinks()?)? .into())
    });

    server.post("/bt/scan", |_: EmptyRequest, _, ctx| {
        check_admin_auth(ctx)?;
        Ok(Json::new(&bt::scan_sinks()?)? .into())
    });

    #[derive(Debug, Clone, Deserialize)]
    struct PairSinkRequest {
        address: String,
    }

    server.post("/bt/pair", |body: PairSinkRequest, _, ctx| {
        check_admin_auth(ctx)?;
        Ok(Json::new(&bt::pair_sink(&body.address)?)? .into())
    });

    server.post("/bt/unpair", |_: EmptyRequest, _, ctx| {
        check_admin_auth(ctx)?;
        Ok(Json::new(&bt::unpair_sink()?)? .into())
    });
}

pub fn load_web(server: &mut HttpServer) {
    let web_build = web_files();

    if let Some(index) = web_build.get_file("index.html") {
        let contents = index.contents();
        server.get("/", move |_, _| {
            Ok(Response {
                status_code: 200,
                content_type: ContentType::Html,
                body: ResponseBody::Bytes(contents),
            })
        });
    }

    fn register_dir(dir: &Dir<'static>, server: &mut HttpServer) {
        for file in dir.files() {
            let route = format!("/{}", file.path().display());

            let contents = file.contents();
            let extension = file
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let content_type = ContentType::from(extension);

            let contents = contents;

            server.get(route, move |_, _| {
                Ok(Response {
                    status_code: 200,
                    content_type: content_type,
                    body: ResponseBody::Bytes(contents),
                })
            });
        }

        for subdir in dir.dirs() {
            register_dir(subdir, server);
        }
    }

    register_dir(&web_build, server);
}
