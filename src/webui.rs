use warp::Filter;
use pioneerfs::network::Network;
use std::sync::{Arc, Mutex};

pub async fn start_webui(network: Arc<Mutex<Network>>) {
    let network_status = warp::path::end().map(move || {
        let network = network.lock().unwrap();
        let status = format!("{:?}", *network); // You can format this as needed
        warp::reply::html(status)
    });

    warp::serve(network_status)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
