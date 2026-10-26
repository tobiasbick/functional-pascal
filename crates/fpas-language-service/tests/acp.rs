//! The nominal ACP unit graph resolves through the generated editor surface.
#![allow(
    clippy::expect_used,
    reason = "editor fixtures require direct public results"
)]
mod support;
use fpas_language_service::LanguageService;
use std::path::Path;
use support::TempDirectory;

#[test]
fn acp_imports_hover_complete_and_navigate_to_the_matching_public_family() {
    let temp = TempDirectory::new("acp-editor");
    let source = "program Consumer;\nuses Std.Acp as Acp, Std.Acp.Types as Types, Std.Acp.Host as Host, Std.Acp.Experimental as Exp, Std.Acp.V2 as V2;\nprocedure Read(Responder: Acp.Responder; Request: Types.ReadTextFileRequest);\nbegin discard Host.ReadTextFile(Responder, Request); end procedure;\nbegin const O := Acp.DefaultOptions(); const P := Exp.ProviderId('p'); const S := V2.SessionId('s'); end.\n";
    let file = temp.write("main.fpas", source);
    let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lib");
    let mut service =
        LanguageService::load_with_standard_library(temp.path(), &library).expect("ACP library");
    for (needle, unit) in [
        ("Acp.DefaultOptions", "Acp.fpas"),
        ("Types.ReadTextFileRequest", "Acp.Types.fpas"),
        ("Host.ReadTextFile", "Acp.Host.fpas"),
        ("Exp.ProviderId", "Acp.Experimental.fpas"),
        ("V2.SessionId", "Acp.V2.fpas"),
    ] {
        let offset =
            source.find(needle).expect("reference") + needle.find('.').expect("qualifier") + 1;
        let hover = service
            .hover(&file, offset)
            .expect("hover")
            .value
            .expect("public declaration");
        assert!(hover.documentation.is_some(), "{needle}");
        let definitions = service
            .definitions(&file, offset)
            .expect("definition")
            .value;
        assert_eq!(definitions.len(), 1, "{needle}: {definitions:?}");
        assert!(
            definitions[0]
                .path
                .ends_with(Path::new("lib/api/Std").join(unit)),
            "{definitions:?}"
        );
        let completions = service
            .completions(&file, offset + 2)
            .expect("completion")
            .value;
        assert!(
            completions.iter().any(|value| value
                .qualified_name
                .ends_with(&needle[needle.find('.').expect("member")..])),
            "{needle}: {completions:?}"
        );
    }
}
