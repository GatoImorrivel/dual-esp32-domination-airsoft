use domination_web::{web_files, Dir};
use serde::Deserialize;

use crate::{
    app::client::AppClient,
    hardware::wifi::WifiConfig,
    http::{server::HttpServer, ContentType, Json, Response, ResponseBody},
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

pub fn load_web(server: &mut HttpServer) {
    let web_build = web_files();

    if let Some(index) = web_build.get_file("index.html") {
        let contents = index.contents();
        server.get("/", move || {
            Ok(Response {
                status_code: 200,
                content_type: ContentType::Html,
                body: ResponseBody::Bytes(contents),
            })
        });
    }

    fn register_dir(dir: &Dir<'static>, server: &mut HttpServer) {
        for file in dir.files() {
            // The file path relative to the root of `dist/`
            let route = format!("/{}", file.path().display());

            let contents = file.contents();
            let extension = file
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let content_type = ContentType::from(extension);

            let contents = contents;

            server.get(route, move || {
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
