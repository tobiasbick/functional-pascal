//! Distributed declarations, reused artifacts and native ACP process ownership.
#![allow(
    clippy::expect_used,
    reason = "integration fixtures fail at public CLI boundaries"
)]
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn distributed_client_runs_independent_stdio_peer_from_source_and_native_image() {
    let root = std::env::temp_dir().join(format!("fpas-acp-distribution-{}", std::process::id()));
    fs::create_dir_all(&root).expect("consumer directory");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (source, target) in [
        ("tests/stdlib/acp/integration_test.fpas", "main.fpas"),
        ("tests/stdlib/acp/v2_test.fpas", "v2.fpas"),
        ("tests/support/acp/scripted_agent.fpas", "peer.fpas"),
    ] {
        fs::copy(repository.join(source), root.join(target)).expect("public fixture");
    }
    fs::write(root.join("consumer.fpas"), "program Consumer; uses Std.Ai.Acp as Acp, Std.Ai.Acp.Types as Types, Std.Ai.Acp.Host as Host, Std.Ai.Acp.Experimental as Exp, Std.Ai.Acp.V2 as V2; begin const O := Acp.DefaultOptions(); const R: Types.RawJson := Types.RawJson(Text := '{}'); const P: Exp.ProviderId := Exp.ProviderId('local'); const S: V2.SessionId := V2.SessionId('s'); end.").expect("all public units");
    fs::write(root.join("client.fpasprj"), "[project]\nname='client'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n").expect("native consumer manifest");
    let fpas = env!("CARGO_BIN_EXE_fpas");
    let command = || {
        let mut value = Command::new(fpas);
        value.current_dir(&root);
        value
    };
    success(
        command()
            .args(["check", "consumer.fpas"])
            .output()
            .expect("consumer check"),
    );
    let documentation = success(
        command()
            .args([
                "doc",
                "Std.Ai.Acp.Host.ReadTextFile",
                "consumer.fpas",
                "--report",
                "json",
            ])
            .output()
            .expect("distributed doc lookup"),
    );
    let documentation: serde_json::Value =
        serde_json::from_slice(&documentation.stdout).expect("doc JSON");
    assert_eq!(
        documentation["matches"][0]["name"],
        "Std.Ai.Acp.Host.ReadTextFile"
    );
    assert_eq!(
        documentation["matches"][0]["handbook"],
        "docs/pascal/std/ai/acp/host.md"
    );
    for source in ["main.fpas", "main.fpas", "v2.fpas"] {
        success(
            command()
                .args(["run", source, "--", fpas, "peer.fpas"])
                .output()
                .expect("source client"),
        );
    }
    success(
        command()
            .args([
                "build",
                "--executable",
                "--name",
                "client",
                "client.fpasprj",
            ])
            .output()
            .expect("native client build"),
    );
    let executable = root.join(if cfg!(windows) {
        "client.exe"
    } else {
        "client"
    });
    // Only the separately executed peer needs sources. The client must use its bundled image.
    fs::remove_file(root.join("main.fpas")).expect("remove client source");
    fs::remove_file(root.join("client.fpasprj")).expect("remove client manifest");
    fs::remove_file(root.join("client.fpascp")).expect("remove external program image");
    success(
        Command::new(executable)
            .current_dir(&root)
            .args([fpas, "peer.fpas"])
            .output()
            .expect("native ACP client"),
    );
    fs::remove_dir_all(root).expect("consumer cleanup after all child processes exited");
}
