//! Independent newline-delimited peer using the production SDK server adapter.

use fpas_mcp::{
    Catalog, Endpoint, Limits, Server,
    model::{CallToolResult, ContentBlock, JsonObject, Tool},
};
use rmcp::ServiceExt;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, ReadHalf, WriteHalf};

pub fn tool() -> Tool {
    let mut tool = Tool::new(
        "update",
        "Update a value after application policy permits it",
        object(json!({
            "type":"object", "properties":{"value":{"type":"integer","minimum":0}},
            "required":["value"], "additionalProperties":false
        })),
    );
    tool.output_schema = Some(Arc::new(object(json!({
        "type":"object", "properties":{"value":{"type":"integer"}},
        "required":["value"], "additionalProperties":false
    }))));
    tool
}

pub fn object(value: Value) -> JsonObject {
    value.as_object().expect("fixture object").clone()
}

pub fn server(limits: Limits) -> Server {
    let mut catalog = Catalog::new(limits).unwrap();
    catalog.register(tool(), 7).unwrap();
    Server::new(catalog)
}

pub fn result(value: Value) -> CallToolResult {
    let mut result = CallToolResult::success(vec![ContentBlock::text("accepted")]);
    result.structured_content = Some(value);
    result
}

pub async fn receive(reader: &mut fpas_mcp::Reader) -> fpas_mcp::Call {
    tokio::time::timeout(Duration::from_secs(5), reader.receive())
        .await
        .expect("application receive deadline")
        .expect("live application reader")
}

pub struct Peer {
    input: WriteHalf<DuplexStream>,
    output: BufReader<ReadHalf<DuplexStream>>,
    service: tokio::task::JoinHandle<()>,
}

impl Peer {
    pub async fn open(endpoint: Endpoint) -> Self {
        let (client, server) = tokio::io::duplex(1024 * 1024);
        let service = tokio::spawn(async move {
            if let Ok(running) = endpoint.serve(server).await {
                let _ = running.waiting().await;
            }
        });
        let (output, input) = tokio::io::split(client);
        let mut peer = Self {
            input,
            output: BufReader::new(output),
            service,
        };
        peer.request(0, "initialize", json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"independent-fixture","version":"1"}})).await;
        let reply = peer.response().await;
        assert_eq!(reply["result"]["protocolVersion"], "2025-06-18");
        peer.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .await;
        peer
    }

    pub async fn send(&mut self, message: Value) {
        let line = serde_json::to_string(&message).unwrap() + "\n";
        self.input.write_all(line.as_bytes()).await.unwrap();
    }

    pub async fn request(&mut self, id: u64, method: &str, params: Value) {
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
            .await;
    }

    pub async fn call(&mut self, id: u64, value: Value) {
        self.request(id, "tools/call", json!({"name":"update","arguments":value}))
            .await;
    }

    pub async fn response(&mut self) -> Value {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let mut line = String::new();
                assert_ne!(
                    self.output.read_line(&mut line).await.unwrap(),
                    0,
                    "SDK output closed"
                );
                let value: Value = serde_json::from_str(&line).expect("JSON-RPC response");
                if value.get("id").is_some() {
                    return value;
                }
            }
        })
        .await
        .expect("SDK response deadline")
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.service.abort();
    }
}
