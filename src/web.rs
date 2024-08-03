use warp::Filter;
use std::sync::{Arc, Mutex};
use pioneerfs::{Network, DebugLevel};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct UploadRequest {
    client_id: String,
    filename: String,
    content: String,
    replication_factor: usize,
}

#[derive(Serialize)]
struct UploadResponse {
    message: String,
    replication_map: Vec<String>,
}

pub async fn start_web_server(network: Arc<Mutex<Network>>) {
    let network_filter = warp::any().map(move || Arc::clone(&network));

    let upload_route = warp::path("upload")
        .and(warp::post())
        .and(warp::body::json())
        .and(network_filter.clone())
        .and_then(handle_upload);

    let status_route = warp::path("status")
        .and(warp::get())
        .and(network_filter.clone())
        .and_then(handle_status);

    let routes = upload_route
        .or(status_route)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_upload(
    req: UploadRequest,
    network: Arc<Mutex<Network>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let client_id = PeerId::from_bytes(&hex::decode(req.client_id).unwrap()).unwrap();
    let filename = req.filename;
    let content = req.content.into_bytes();
    let replication_factor = req.replication_factor;

    let mut network = network.lock().unwrap();
    match network.upload_file(&client_id, filename, content, replication_factor) {
        Ok(stored_nodes) => {
            let replication_map: Vec<String> = stored_nodes.iter().map(|id| id.to_string()).collect();
            Ok(warp::reply::json(&UploadResponse {
                message: "File uploaded successfully".to_string(),
                replication_map,
            }))
        }
        Err(e) => Ok(warp::reply::json(&UploadResponse {
            message: format!("Failed to upload file: {}", e),
            replication_map: vec![],
        })),
    }
}
async fn handle_status(
    network: Arc<Mutex<Network>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let network = network.lock().unwrap();
    let status = network.get_network_status();
    Ok(warp::reply::json(&status))
}
