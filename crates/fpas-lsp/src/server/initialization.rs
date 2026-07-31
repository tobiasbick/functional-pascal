//! Validated workspace and standard-library initialization paths.

use std::path::PathBuf;

use tower_lsp_server::ls_types::{InitializeParams, Uri};

use crate::convert::file_uri_to_path;

pub(super) struct InitializationPaths {
    pub(super) workspace_root: Option<PathBuf>,
    pub(super) standard_library_root: Option<PathBuf>,
}

impl InitializationPaths {
    pub(super) fn from_params(params: &InitializeParams) -> Result<Self, String> {
        let workspace_root = initialization_root_uri(params)
            .map(file_uri_to_path)
            .transpose()
            .map_err(|error| error.to_string())?;
        let standard_library_root = standard_library_uri(params)?
            .as_ref()
            .map(file_uri_to_path)
            .transpose()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            workspace_root,
            standard_library_root,
        })
    }
}

#[expect(
    deprecated,
    reason = "LSP 3.17 permits rootUri when workspaceFolders is unavailable"
)]
fn initialization_root_uri(params: &InitializeParams) -> Option<&Uri> {
    params
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .map(|folder| &folder.uri)
        .or(params.root_uri.as_ref())
}

fn standard_library_uri(params: &InitializeParams) -> Result<Option<Uri>, String> {
    let Some(options) = params.initialization_options.as_ref() else {
        return Ok(None);
    };
    let Some(value) = options.get("standardLibraryUri") else {
        return Ok(None);
    };
    let text = value.as_str().ok_or_else(|| {
        String::from("Initialization option `standardLibraryUri` must be a file-URI string.")
    })?;
    text.parse::<Uri>().map(Some).map_err(|error| {
        format!("Initialization option `standardLibraryUri` is not a valid URI: {error}")
    })
}
