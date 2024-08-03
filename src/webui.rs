use warp::Filter;
use pioneerfs::network::Network;
use serde_json::json;
use std::sync::{Arc, Mutex};

pub async fn start_webui(network: Arc<Mutex<Network>>) {
    let network_status = warp::path::end().map(move || {
        let network = network.lock().unwrap();
        let status = json!({
            "clients": network.clients.len(),
            "storage_nodes": network.storage_nodes.len(),
            "deals": network.deals.len(),
            "marketplace": network.marketplace.len(),
        }).to_string();
        warp::reply::html(status)
    });

    warp::serve(network_status)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
