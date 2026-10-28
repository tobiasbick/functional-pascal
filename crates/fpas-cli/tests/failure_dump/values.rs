use super::*;

#[test]
fn nested_fault_values_preserve_integer_extremes_scopes_and_dictionary_key_values() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", "program Main; var Global:integer:=7; procedure Leaf(Argument:integer); begin const Minimum := -9223372036854775808; const Maximum:=9223372036854775807; const Unicode:='€😀'; const Mapping:dict of integer to string:=[7:'seven', 9:'nine']; const Shadow:=11; begin const Shadow:=13; panic('typed'); end; end procedure; procedure Caller(); begin const CallerValue:=19; Leaf(23); panic('unreachable'); end procedure; begin Global:=31; Caller(); panic('unreachable'); end.");
    let output = fixture.run(&["run", "main.fpas", "--failure-dump", "report.json"]);
    exit(&output, 2);
    let report = fixture.report("report.json");
    let frames = report["tasks"][0]["frames"].as_array().expect("frames");
    assert_eq!(frames.len(), 3, "{report}");
    let bindings = frames[0]["bindings"].as_array().expect("bindings");
    let binding = |name| {
        bindings
            .iter()
            .find(|binding| {
                binding["name"]
                    .as_str()
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(name))
            })
            .expect("named binding")
    };
    assert_eq!(
        binding("Minimum")["value"]["scalar"],
        "-9223372036854775808"
    );
    assert_eq!(binding("Maximum")["value"]["scalar"], "9223372036854775807");
    assert_eq!(binding("Unicode")["value"]["scalar"], "€😀");
    assert_eq!(binding("Argument")["kind"], "parameter");
    assert_eq!(binding("Argument")["value"]["scalar"], "23");
    let shadows: Vec<_> = bindings
        .iter()
        .filter(|binding| {
            binding["name"]
                .as_str()
                .expect("name")
                .eq_ignore_ascii_case("Shadow")
        })
        .collect();
    assert_eq!(shadows.len(), 2);
    assert_ne!(shadows[0]["scope_id"], shadows[1]["scope_id"]);
    let mapping = &binding("Mapping")["value"];
    assert_eq!(mapping["kind"], "dict");
    assert_eq!(mapping["total_children"], 4);
    assert_eq!(mapping["children"][0]["name"], "[0].key");
    assert_eq!(mapping["children"][0]["value"]["scalar"], "7");
    assert_eq!(mapping["children"][1]["name"], "[0].value");
    assert_eq!(mapping["children"][1]["value"]["scalar"], "seven");
    assert_eq!(mapping["children"][2]["value"]["scalar"], "9");
    assert_eq!(report["globals"][0]["value"]["scalar"], "31");
    assert_eq!(report["globals"][0]["kind"], "global");
    assert!(report["globals"][0]["scope_id"].is_null());
    assert!(report["error"]["location"]["line"].as_u64().expect("line") >= 1);
}

#[test]
fn images_without_binding_metadata_keep_stored_positions_without_invented_locals() {
    let fixture = Fixture::new();
    let (program, errors) =
        fpas_parser::parse("program Bare; begin const Secret:=23; panic('bare'); end.");
    assert!(errors.is_empty());
    let mut executable = fpas_compiler::compile(&program)
        .expect("compile bare fixture")
        .into_unverified();
    for function in &mut executable.functions {
        function.debug = Default::default();
    }
    let executable = executable
        .verify()
        .expect("metadata-free executable is valid");
    let image = fpas_program::ProgramImage::new(
        fpas_program::ProgramIdentity {
            contracts_enabled: true,
            compiler_version: env!("CARGO_PKG_VERSION").into(),
            bytecode_version: fpas_runtime::BYTECODE_VERSION,
            source_hash: fpas_program::Digest::of(b"bare"),
            options_hash: fpas_program::Digest::of(b"bare-options"),
            units: Vec::new(),
        },
        vec!["bare.fpas".into()],
        vec![fpas_program::Digest::of(b"bare")],
        executable,
    )
    .expect("metadata-free image");
    fs::write(
        fixture.0.join("bare.fpascp"),
        fpas_program::encode(&image).expect("encode image"),
    )
    .expect("write image");
    exit(
        &fixture.run(&["run", "bare.fpascp", "--failure-dump", "report.json"]),
        2,
    );
    let report = fixture.report("report.json");
    assert_eq!(report["error"]["message"], "panic: bare");
    assert_eq!(report["error"]["location"]["line"], 1);
    assert_eq!(
        report["sources"],
        serde_json::json!([{"id":0,"path":"bare.fpas"}])
    );
    assert_eq!(report["capture"]["incomplete"], true);
    assert!(
        report["capture"]["reasons"]
            .as_array()
            .expect("reasons")
            .contains(&Value::String("missing_metadata".into()))
    );
    for frame in report["tasks"][0]["frames"].as_array().expect("frames") {
        assert_eq!(frame["location"], report["error"]["location"]);
        assert_eq!(frame["bindings"], serde_json::json!([]));
    }
}

#[test]
fn structured_secondaries_keep_real_task_identities_and_share_sorted_task_entries() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", "program Secondary; uses Std.Tasks as Tasks; begin const Ready:channel of integer:=Tasks.CreateChannel(1); const Group:=Tasks.CreateTaskGroup(); discard Tasks.StartTaskInGroup(Group, procedure(Token:Tasks.CancellationToken) begin const ChildLocal:=37; discard Tasks.Send(Ready,1); panic('child'); end procedure); discard Tasks.Receive(Ready); const RootLocal:=19; panic('body'); end.");
    exit(
        &fixture.run(&["run", "main.fpas", "--failure-dump", "report.json"]),
        2,
    );
    let report = fixture.report("report.json");
    assert_eq!(report["error"]["message"], "panic: body");
    assert_eq!(report["error"]["task_id"], 0);
    assert_eq!(report["related_errors"][0]["message"], "panic: child");
    let child = report["related_errors"][0]["task_id"]
        .as_u64()
        .expect("child id");
    assert_ne!(child, 0);
    let tasks = report["tasks"].as_array().expect("task catalog");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["id"], 0);
    assert_eq!(tasks[1]["id"], child);
    assert_eq!(tasks[1]["observation"], "fault");
    assert!(
        tasks[1]["frames"][0]["bindings"]
            .as_array()
            .expect("child bindings")
            .iter()
            .any(|binding| binding["name"] == "ChildLocal" && binding["value"]["scalar"] == "37")
    );
}

#[test]
fn source_name_budget_keeps_known_diagnostic_coordinates() {
    let fixture = Fixture::new();
    let (program, errors) =
        fpas_parser::parse("program ManySources; begin panic('known position'); end.");
    assert!(errors.is_empty());
    let mut executable = fpas_compiler::compile(&program)
        .expect("compile fixture")
        .into_unverified();
    let paths: Vec<_> = (0..300)
        .map(|index| format!("unit{index:03}/{}.fpas", "a".repeat(4080)))
        .collect();
    let mut strings: Vec<_> = executable.strings.iter().map(str::to_owned).collect();
    executable.source_map.sources = paths
        .iter()
        .map(|path| {
            let id =
                fpas_runtime::StringId::try_from_index(strings.len()).expect("source string id");
            strings.push(path.clone());
            id
        })
        .collect();
    executable.strings = fpas_runtime::StringTable::new(strings);
    for run in &mut executable.source_map.runs {
        run.source = fpas_runtime::SourceId::try_from_index(299).expect("source id");
    }
    let image = fpas_program::ProgramImage::new(
        fpas_program::ProgramIdentity {
            contracts_enabled: true,
            compiler_version: env!("CARGO_PKG_VERSION").into(),
            bytecode_version: fpas_runtime::BYTECODE_VERSION,
            source_hash: fpas_program::Digest::of(b"many sources"),
            options_hash: fpas_program::Digest::of(b"many source options"),
            units: Vec::new(),
        },
        paths,
        vec![fpas_program::Digest::of(b"source"); 300],
        executable.verify().expect("valid source map"),
    )
    .expect("many-source image");
    fs::write(
        fixture.0.join("many.fpascp"),
        fpas_program::encode(&image).expect("encode image"),
    )
    .expect("write image");
    exit(
        &fixture.run(&["run", "many.fpascp", "--failure-dump", "report.json"]),
        2,
    );
    let report = fixture.report("report.json");
    assert_eq!(report["error"]["message"], "panic: known position");
    assert_eq!(report["error"]["location"]["source_id"], 299);
    assert_eq!(report["error"]["location"]["line"], 1);
    assert!(
        report["error"]["location"]["column"]
            .as_u64()
            .expect("known column")
            > 1
    );
    let source = report["sources"]
        .as_array()
        .expect("source identities")
        .iter()
        .find(|source| source["id"] == 299)
        .expect("known identity");
    assert!(source["path"].is_null());
    assert!(
        report["capture"]["reasons"]
            .as_array()
            .expect("reasons")
            .contains(&Value::String("retained_byte_limit".into()))
    );
}
