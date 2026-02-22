use serde::Deserialize;

use crate::{
    app::client::AppClient,
    hardware::wifi::WifiConfig,
    http::server::{HttpServer, Json, Response},
};

pub fn routes(server: &mut HttpServer) {
    #[derive(Debug, Clone, Copy, Deserialize)]
    struct EmptyRequest {}

    server.get("/game/progress", || {
        let client = AppClient::get();
        let progress = client.get_match_progress()?;
        Ok(Json::new(&progress)?.into())
    });

    server.post("/game/start", |_: EmptyRequest| {
        let client = AppClient::get();
        client.start_game()?;
        Ok(Response::ok())
    });

    server.post("/game/stop", |_: EmptyRequest| {
        let client = AppClient::get();
        client.stop_game()?;
        Ok(Response::ok())
    });

    #[derive(Debug, Clone, Deserialize)]
    struct ConfigureRequest {
        wifi_config: WifiConfig,
    }

    server.post("/app/configure", |request: ConfigureRequest| {
        let client = AppClient::get();
        client.setup_wifi(request.wifi_config)?;
        Ok(Response::ok())
    });

    server.get("/app/status", || {
        let client = AppClient::get();
        let app_status = client.get_app_state()?;
        Ok(Json::new(&app_status)?.into())
    });
}
