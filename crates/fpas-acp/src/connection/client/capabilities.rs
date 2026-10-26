use super::super::Fault;
use crate::{RawJson, protocol::v1 as sdk};

pub(super) fn require(supported: bool, name: &'static str) -> Result<(), Fault> {
    supported
        .then_some(())
        .ok_or(Fault::UnsupportedCapability(name))
}

pub(super) fn setup(caps: &sdk::AgentCapabilities, raw: &RawJson) -> Result<(), Fault> {
    if let Some(roots) = raw.member("additionalDirectories")?
        && roots.text() != "null"
        && roots.text() != "[]"
    {
        require(
            caps.session_capabilities.additional_directories.is_some(),
            "AdditionalDirectories",
        )?;
    }
    if let Some(servers) = raw.member("mcpServers")? {
        if servers.text() == "null" {
            return Ok(());
        }
        let servers: Vec<sdk::McpServer> = crate::codec::deserialize(&servers)?;
        for server in servers {
            match server {
                sdk::McpServer::Http(_) => require(caps.mcp_capabilities.http, "McpHttp")?,
                sdk::McpServer::Sse(_) => require(caps.mcp_capabilities.sse, "McpSse")?,
                sdk::McpServer::Stdio(_) => {}
                sdk::McpServer::Acp(_) => require(caps.mcp_capabilities.acp, "McpOverAcp")?,
                _ => return Err(Fault::UnsupportedCapability("ExperimentalMcp")),
            }
        }
    }
    Ok(())
}

pub(super) fn prompt(caps: &sdk::PromptCapabilities, raw: &RawJson) -> Result<(), Fault> {
    let prompt: sdk::PromptRequest = crate::codec::deserialize(raw)?;
    for block in prompt.prompt {
        match block {
            sdk::ContentBlock::Image(_) => require(caps.image, "PromptImage")?,
            sdk::ContentBlock::Audio(_) => require(caps.audio, "PromptAudio")?,
            sdk::ContentBlock::Resource(_) => {
                require(caps.embedded_context, "PromptEmbeddedContext")?
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn incoming(caps: &sdk::ClientCapabilities, method: &str, raw: &RawJson) -> bool {
    match method {
        "fs/read_text_file" => caps.fs.read_text_file,
        "fs/write_text_file" => caps.fs.write_text_file,
        method if method.starts_with("terminal/") => caps.terminal,
        "elicitation/create" => {
            let Ok(request) = crate::codec::deserialize::<sdk::CreateElicitationRequest>(raw)
            else {
                return false;
            };
            caps.elicitation
                .as_ref()
                .is_some_and(|caps| match request.mode {
                    sdk::ElicitationMode::Form(_) => caps.form.is_some(),
                    sdk::ElicitationMode::Url(_) => caps.url.is_some(),
                    _ => false,
                })
        }
        _ => true,
    }
}
