use warp::{Filter, ws::Message, ws::WebSocket};
use futures::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use pioneerfs::network::Network;
use serde_json::json;
use std::sync::{Arc, Mutex};

pub async fn start_webui(network: Arc<Mutex<Network>>, tx: broadcast::Sender<String>) {
    let network_status = {
        let network = Arc::clone(&network);
        warp::path("status").map(move || {
            let network = network.lock().unwrap();
            let status = json!({
                "clients": network.clients.keys().collect::<Vec<_>>(),
                "storage_nodes": network.storage_nodes.keys().collect::<Vec<_>>(),
                "deals": network.deals.len(),
                "marketplace": network.marketplace.len(),
            }).to_string();
            warp::reply::json(&status)
        })
    };

    let index = warp::path::end().map(|| {
        warp::reply::html(INDEX_HTML)
    });

    let run_tests = {
        let network = Arc::clone(&network);
        let tx_clone = tx.clone();
        warp::path("run_tests").map(move || {
            let network = network.lock().unwrap();
            crate::run_advanced_network_tests(&network, tx_clone.clone());
            warp::reply::html("Advanced network tests started")
        })
    };

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_broadcast(tx.clone()))
        .map(|ws: warp::ws::Ws, tx| {
            ws.on_upgrade(move |socket| handle_socket(socket, tx))
        });

    warp::serve(network_status.or(index).or(run_tests).or(ws_route))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn with_broadcast(
    tx: broadcast::Sender<String>,
) -> impl Filter<Extract = (broadcast::Sender<String>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

async fn handle_socket(ws: WebSocket, tx: broadcast::Sender<String>) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = tx.subscribe();

    tokio::task::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_tx.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(result) = ws_rx.next().await {
        if result.is_err() {
            break;
        }
    }
}

const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Network Map</title>
    <script src="https://d3js.org/d3.v6.min.js"></script>
    <style>
        .node {
            stroke: #fff;
            stroke-width: 1.5px;
        }

        .link {
            stroke: #999;
            stroke-opacity: 0.6;
        }
    </style>
</head>
<body>
    <h1>Network Map</h1>
    <svg width="960" height="600"></svg>
    <button id="run-tests-button">Run Advanced Network Tests</button>
    <div id="test-result"></div>
    <script>
        const ws = new WebSocket(`ws://${window.location.host}/ws`);
        ws.onmessage = (event) => {
            const message = event.data;
            const resultDiv = document.getElementById("test-result");
            resultDiv.innerText += message + "\n";
        };
        document.getElementById("run-tests-button").addEventListener("click", () => {
            fetch("/run_tests")
                .then(response => response.text())
                .then(data => {
                    document.getElementById("test-result").innerText = data;
                })
                .catch(error => {
                    document.getElementById("test-result").innerText = "Error: " + error;
                });
        });
        const width = 960;
        const height = 600;

        const svg = d3.select("svg")
            .attr("width", width)
            .attr("height", height);

        const simulation = d3.forceSimulation()
            .force("link", d3.forceLink().id(d => d.id))
            .force("charge", d3.forceManyBody())
            .force("center", d3.forceCenter(width / 2, height / 2));

        d3.json("/status").then(data => {
            if (!data || !data.clients || !data.storage_nodes) {
                console.error("Invalid data received from /status endpoint", data);
                // Draw an empty canvas
                const graph = {
                    nodes: [],
                    links: []
                };

                const link = svg.append("g")
                    .attr("class", "links")
                    .selectAll("line")
                    .data(graph.links)
                    .enter().append("line")
                    .attr("class", "link");

                const node = svg.append("g")
                    .attr("class", "nodes")
                    .selectAll("circle")
                    .data(graph.nodes)
                    .enter().append("circle")
                    .attr("class", "node")
                    .attr("r", 5)
                    .attr("fill", "none");

                simulation
                    .nodes(graph.nodes)
                    .on("tick", ticked);

                simulation.force("link")
                    .links(graph.links);

                function ticked() {
                    link
                        .attr("x1", d => d.source.x)
                        .attr("y1", d => d.source.y)
                        .attr("x2", d => d.target.x)
                        .attr("y2", d => d.target.y);

                    node
                        .attr("cx", d => d.x)
                        .attr("cy", d => d.y);
                }
                return;
            }
            const graph = {
                nodes: data.clients.map(id => ({ id, group: "client" }))
                    .concat(data.storage_nodes.map(id => ({ id, group: "storage_node" }))),
                links: [] // Add links if needed
            };

            const link = svg.append("g")
                .attr("class", "links")
                .selectAll("line")
                .data(graph.links)
                .enter().append("line")
                .attr("class", "link");

            const node = svg.append("g")
                .attr("class", "nodes")
                .selectAll("circle")
                .data(graph.nodes)
                .enter().append("circle")
                .attr("class", "node")
                .attr("r", 5)
                .attr("fill", d => d.group === "client" ? "blue" : "green")
                .call(drag(simulation));

            node.append("title")
                .text(d => d.id);

            simulation
                .nodes(graph.nodes)
                .on("tick", ticked);

            simulation.force("link")
                .links(graph.links);

            function ticked() {
                link
                    .attr("x1", d => d.source.x)
                    .attr("y1", d => d.source.y)
                    .attr("x2", d => d.target.x)
                    .attr("y2", d => d.target.y);

                node
                    .attr("cx", d => d.x)
                    .attr("cy", d => d.y);
            }

            function drag(simulation) {
                function dragstarted(event, d) {
                    if (!event.active) simulation.alphaTarget(0.3).restart();
                    d.fx = d.x;
                    d.fy = d.y;
                }

                function dragged(event, d) {
                    d.fx = event.x;
                    d.fy = event.y;
                }

                function dragended(event, d) {
                    if (!event.active) simulation.alphaTarget(0);
                    d.fx = null;
                    d.fy = null;
                }

                return d3.drag()
                    .on("start", dragstarted)
                    .on("drag", dragged)
                    .on("end", dragended);
            }
        });
    </script>
</body>
</html>
"#;
