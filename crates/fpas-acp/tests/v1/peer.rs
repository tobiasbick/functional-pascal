use fpas_acp::{
    RawJson,
    connection::{Connection, Event, Limits, Payload, Reader, Runtime, Settings},
    schema::Version,
    transport::MessageEndpoint,
};
use std::time::Duration;

pub fn raw(text: &str) -> RawJson {
    RawJson::parse(text).unwrap()
}
pub fn wait<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap()
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(10), future)
                .await
                .expect("peer must progress")
        })
}
pub struct Peer {
    pub connection: Connection,
    pub endpoint: MessageEndpoint,
    pub reader: Reader,
    pub offer: RawJson,
    _runtime: Runtime,
}
impl Peer {
    pub fn new() -> Self {
        Self::configured(Settings::for_version(Version::V1), Limits::default())
    }
    pub fn configured(settings: Settings, limits: Limits) -> Self {
        let offer = if settings.requested_version().unwrap() == Version::V2 {
            raw(&serde_json::json!({"protocolVersion":2,"info":settings.client_info,"capabilities":settings.v2_client_capabilities}).to_string())
        } else {
            raw(&serde_json::to_string(
                &fpas_acp::protocol::v1::InitializeRequest::new(
                    fpas_acp::protocol::ProtocolVersion::V1,
                )
                .client_capabilities(settings.client_capabilities.clone())
                .client_info(settings.client_info.clone()),
            )
            .unwrap())
        };
        let runtime = Runtime::new().unwrap();
        let (connection, endpoint) = runtime
            .connect_messages_configured(settings, limits)
            .unwrap();
        let reader = connection.reader().unwrap();
        Self {
            connection,
            endpoint,
            reader,
            offer,
            _runtime: runtime,
        }
    }
    pub fn take(&self) -> RawJson {
        wait(async {
            loop {
                if let Some(message) = self.endpoint.try_take().unwrap() {
                    return message;
                }
                assert!(!self.endpoint.output_closed(), "unexpected endpoint close");
                tokio::task::yield_now().await;
            }
        })
    }
    pub fn feed(&self, message: RawJson) {
        wait(async {
            while !self.endpoint.feed(message.clone()).unwrap() {
                tokio::task::yield_now().await;
            }
        });
    }
    pub fn event(&self) -> Event {
        wait(self.reader.receive()).unwrap()
    }
    pub fn response(&self, request: &RawJson, result: &str) {
        let id = request.member("id").unwrap().unwrap();
        self.feed(raw(&format!(
            r#"{{"jsonrpc":"2.0","id":{},"result":{result}}}"#,
            id.text()
        )));
    }
    pub fn exchange(&self, method: &str, params: &str, result: &str) -> Event {
        let token = self.connection.send(method, raw(params)).unwrap();
        let request = self.take();
        assert_eq!(
            request.member("method").unwrap().unwrap(),
            raw(&format!("\"{method}\""))
        );
        assert_eq!(request.member("params").unwrap().unwrap(), raw(params));
        self.response(&request, result);
        let event = self.event();
        assert!(
            matches!(event.payload, Payload::Response(actual, Version::V1, _) if actual == token),
            "{:?}",
            event.payload
        );
        assert_eq!(
            event
                .raw
                .as_ref()
                .unwrap()
                .member("result")
                .unwrap()
                .unwrap(),
            raw(result)
        );
        event
    }
    pub fn initialize(&self, result: &str) {
        self.exchange("initialize", self.offer.text(), result);
    }
    pub fn session(&self, id: &str) {
        self.exchange(
            "session/new",
            r#"{"cwd":"/workspace","mcpServers":[]}"#,
            &format!(r#"{{"sessionId":"{id}"}}"#),
        );
    }
    pub fn update(&self, session: &str, update: &str) -> RawJson {
        let frame = raw(&format!(
            r#"{{"jsonrpc":"2.0","method":"session/update","params":{{"sessionId":"{session}","update":{update}}}}}"#
        ));
        self.feed(frame.clone());
        frame
    }
}
