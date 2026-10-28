use fpas_acp::{RawJson, connection::Fault, protocol::v1 as sdk};
use std::path::Path;

pub(super) fn validate(method: &str, raw: &RawJson) -> Result<(), Fault> {
    let path = raw.member("path")?.ok_or(Fault::InvalidArgument("Path"))?;
    let path: String =
        serde_json::from_str(path.text()).map_err(|_| Fault::InvalidArgument("Path"))?;
    if !Path::new(&path).is_absolute() || path.contains('\0') {
        return Err(Fault::InvalidArgument("Path"));
    }
    if method == "fs/read_text_file" {
        let request = fpas_acp::codec::decode::<sdk::ReadTextFileRequest>(raw.clone())?.value;
        if request.line == Some(0) {
            return Err(Fault::InvalidArgument("Line"));
        }
    }
    Ok(())
}
pub(super) fn execute(method: &str, raw: &RawJson, maximum: usize) -> Result<RawJson, Fault> {
    if method == "fs/write_text_file" {
        let request = fpas_acp::codec::decode::<sdk::WriteTextFileRequest>(raw.clone())?.value;
        crate::stdlib::write_text_for_host(&request.path.to_string_lossy(), &request.content)
            .map_err(|_| Fault::Host)?;
        return RawJson::parse("{}").map_err(Into::into);
    }
    let request = fpas_acp::codec::decode::<sdk::ReadTextFileRequest>(raw.clone())?.value;
    let text = crate::stdlib::read_text_for_host(&request.path.to_string_lossy())
        .map_err(|_| Fault::Host)?;
    let start = request.line.unwrap_or(1).saturating_sub(1) as usize;
    let count = request.limit.map_or(usize::MAX, |value| value as usize);
    let mut content = String::new();
    // Preserve CRLF/LF and a missing final terminator; an empty range stays empty.
    for line in text.split_inclusive('\n').skip(start).take(count) {
        if content.len().saturating_add(line.len()) > maximum {
            return Err(Fault::LimitExceeded("MessageBytes", maximum));
        }
        content.push_str(line);
    }
    let encoded =
        serde_json::to_string(&sdk::ReadTextFileResponse::new(content)).map_err(|_| Fault::Host)?;
    if encoded.len().saturating_add(128) > maximum {
        return Err(Fault::LimitExceeded("MessageBytes", maximum));
    }
    RawJson::parse(&encoded).map_err(Into::into)
}
