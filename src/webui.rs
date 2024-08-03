use warp::Filter;

pub async fn start_webui() {
    let hello = warp::path::end().map(|| warp::reply::html("Hello, WebUI!"));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
