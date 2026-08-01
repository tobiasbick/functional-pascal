#![allow(
    clippy::expect_used,
    reason = "navigation fixtures use explicit source offsets and readable assertions"
)]

mod support;

use fpas_language_service::{
    HighlightKind, LanguageService, SymbolKind, WORKSPACE_SYMBOL_LIMIT, WorkspaceContext,
};
use support::TempDirectory;

#[test]
fn workspace_symbols_filter_rank_limit_and_preserve_equal_names() {
    let temp = TempDirectory::new("workspace-symbol-query");
    let manifest = temp.write(
        "symbols.fpasprj",
        "[project]\nname = \"symbols\"\nkind = \"library\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "src/first.fpas",
        "unit Demo.First;\n\npublic function Create(): integer; begin return 1 end;\nfunction LocalCreate(): integer; begin return 2 end;\n",
    );
    temp.write(
        "src/second.fpas",
        "unit Demo.Second;\n\npublic function Create(): integer; begin return 3 end;\n",
    );
    let mut service = LanguageService::load(&manifest);

    let create = service.workspace_symbols("create").expect("Create query");

    assert_eq!(
        create
            .iter()
            .filter(|location| location.symbol.name == "Create")
            .count(),
        2,
        "{create:?}"
    );
    assert_eq!(create[0].symbol.name, "Create", "{create:?}");
    assert!(
        create
            .iter()
            .any(|location| location.symbol.name == "LocalCreate"),
        "{create:?}"
    );

    let all = service.workspace_symbols("").expect("empty query");
    assert!(all.len() <= WORKSPACE_SYMBOL_LIMIT, "{}", all.len());
    let again = service.workspace_symbols("").expect("stable empty query");
    assert_eq!(all, again);
}

#[test]
fn workspace_symbols_are_bounded_and_include_unsaved_local_declarations() {
    let temp = TempDirectory::new("workspace-symbol-limit");
    let declarations = (0..130)
        .map(|index| format!("var Item{index:03}: integer := {index};\n"))
        .collect::<String>();
    let source = format!("program Many;\n\n{declarations}\nbegin\nend.\n");
    let path = temp.write("many.fpas", &source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    service
        .documents_mut()
        .open_document(&path, 1, source.replace("Item129", "UnsavedItem"))
        .expect("unsaved source");

    let matches = service.workspace_symbols("item").expect("bounded query");

    assert_eq!(matches.len(), WORKSPACE_SYMBOL_LIMIT);
    assert!(
        service
            .workspace_symbols("Unsaved")
            .expect("unsaved query")
            .iter()
            .any(|location| location.symbol.name == "UnsavedItem")
    );
}

#[test]
fn document_highlights_respect_shadowing_and_classify_writes() {
    let temp = TempDirectory::new("document-highlights");
    let source = r#"program Highlights;

type Holder = record
  public Item: integer;
end;

mutable var Value: integer := 1;
mutable var Pair: Holder := record Item := Value; end;

function Read(Value: integer): integer;
begin
  // Value is ignored
  var Text: string := 'Value';
  return Value
end;

begin
  Value := Value + 1;
  Pair.Item := Value
end.
"#;
    let path = temp.write("highlights.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let global_write = source.rfind("Value :=").expect("global write");
    let highlights = service
        .document_highlights(&path, global_write)
        .expect("global highlights")
        .value;

    assert_eq!(highlights.len(), 5, "{highlights:?}");
    assert_eq!(highlights[0].kind, HighlightKind::Declaration);
    assert_eq!(highlights[2].kind, HighlightKind::Write);
    assert!(
        highlights.iter().all(|highlight| highlight.span.offset
            != source.find("Value is ignored").expect("comment text"))
    );

    let parameter_use = source.find("return Value").expect("parameter use") + "return ".len();
    let parameter = service
        .document_highlights(&path, parameter_use)
        .expect("parameter highlights")
        .value;
    assert_eq!(parameter.len(), 2, "{parameter:?}");

    let pair_write = source.find("Pair.Item :=").expect("member assignment");
    let pair = service
        .document_highlights(&path, pair_write)
        .expect("record base highlights")
        .value;
    assert_eq!(pair.len(), 2, "{pair:?}");
    assert_eq!(pair[1].kind, HighlightKind::Write);
}

#[test]
fn type_definition_follows_imported_aliases_members_parameters_and_results() {
    let temp = TempDirectory::new("type-definition");
    let manifest = temp.write(
        "types.fpasprj",
        "[project]\nname = \"types\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let unit = temp.write(
        "src/types.fpas",
        r#"unit Demo.Types;

public type Point = record
  public X: integer;
end;

public type PointAlias = Point;

public type Holder = record
  public Item: Point;
  public property Selected: Point read Item;
end;

type Secret = Point;

public function Echo(Value: Point): Point;
begin
  return Value
end;
"#,
    );
    let main_source = r#"program TypesApp;

uses Demo.Types;

begin
  var AliasValue: PointAlias := record X := 1; end;
  var HolderValue: Holder := record Item := AliasValue; end;
  var PointValue: Point := HolderValue.Item;
  var SelectedValue: Point := HolderValue.Selected;
  var ResultValue: Point := Demo.Types.Echo(PointValue);
  var HiddenValue: Secret := PointValue
end.
"#;
    let main = temp.write("src/main.fpas", main_source);
    let mut service = LanguageService::load(&manifest);

    let alias_value = main_source.find("AliasValue:").expect("alias variable");
    let alias_target = service
        .type_definitions(&main, alias_value)
        .expect("alias variable type")
        .value;
    assert_eq!(alias_target[0].symbol.name, "PointAlias");

    let member = main_source.find("HolderValue.Item").expect("member") + "HolderValue.".len();
    let member_target = service
        .type_definitions(&main, member)
        .expect("member type")
        .value;
    assert_eq!(member_target[0].symbol.name, "Point");

    let property =
        main_source.find("HolderValue.Selected").expect("property") + "HolderValue.".len();
    let property_target = service
        .type_definitions(&main, property)
        .expect("property type")
        .value;
    assert_eq!(property_target[0].symbol.name, "Point");

    let unit_source = std::fs::read_to_string(&unit).expect("unit source");
    let alias_decl = unit_source.find("PointAlias =").expect("alias declaration");
    let alias_definition = service
        .type_definitions(&unit, alias_decl)
        .expect("alias target")
        .value;
    assert_eq!(alias_definition[0].symbol.name, "Point");

    let parameter_use = unit_source.find("return Value").expect("parameter use") + "return ".len();
    let parameter_target = service
        .type_definitions(&unit, parameter_use)
        .expect("parameter type")
        .value;
    assert_eq!(parameter_target[0].symbol.name, "Point");

    let call = main_source
        .find("Demo.Types.Echo(PointValue)")
        .expect("qualified function call")
        + "Demo.Types.".len();
    let result_target = service
        .type_definitions(&main, call)
        .expect("routine result type")
        .value;
    assert_eq!(result_target[0].symbol.name, "Point");
    assert_eq!(result_target[0].path, unit);

    let hidden = main_source
        .find("HiddenValue:")
        .expect("private type variable");
    assert!(
        service
            .type_definitions(&main, hidden)
            .expect("private type remains inaccessible")
            .value
            .is_empty()
    );
}

#[test]
fn unknown_type_definition_and_malformed_selection_are_safe() {
    let temp = TempDirectory::new("navigation-negative-selection");
    let source = "program Broken;\n\nbegin\n  var Music: string := '𝄞';\n  Missing(\nend.";
    let path = temp.write("broken.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let missing = source.find("Missing").expect("unknown symbol");

    assert!(
        service
            .type_definitions(&path, missing)
            .expect("unknown type definition")
            .value
            .is_empty()
    );
    let (_, ranges) = service
        .selection_ranges(&path, &[missing])
        .expect("malformed selection");
    assert_eq!(ranges[0].span.length, "Missing".len());
    assert!(ranges[0].parent.is_none(), "{:?}", ranges[0]);
}

#[test]
fn selection_ranges_expand_from_identifier_through_statement_and_unit() {
    let temp = TempDirectory::new("selection-ranges");
    let source = r#"program Select;

function Read(Value: integer): integer;
begin
  if Value > 0 then
  begin
    return Value
  end
  else
    return 0
end;

begin
  var ResultValue: integer := Read(1)
end.
"#;
    let path = temp.write("select.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let value = source.find("return Value").expect("return value") + "return ".len();

    let (_, ranges) = service
        .selection_ranges(&path, &[value])
        .expect("selection ranges");
    let mut chain = Vec::new();
    let mut current = Some(&ranges[0]);
    while let Some(range) = current {
        chain.push(range.span);
        current = range.parent.as_deref();
    }

    assert!(chain.len() >= 5, "{chain:?}");
    assert_eq!(
        &source[chain[0].offset..chain[0].offset + chain[0].length],
        "Value"
    );
    assert_eq!(
        chain.last().expect("compilation range").length,
        source.trim_end().len()
    );
    assert!(chain.windows(2).all(|pair| {
        pair[1].offset <= pair[0].offset
            && pair[0].offset + pair[0].length <= pair[1].offset + pair[1].length
    }));
}

#[test]
fn workspace_symbol_kinds_remain_editor_facing() {
    let temp = TempDirectory::new("workspace-symbol-kind");
    let path = temp.write(
        "kind.fpas",
        "program Kinds; begin var Value: integer := 1 end.",
    );
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    service
        .documents_mut()
        .open_document(
            &path,
            1,
            "program Kinds; begin var Value: integer := 1 end.",
        )
        .expect("open loose document");

    let symbols = service.workspace_symbols("Value").expect("symbol kind");

    assert_eq!(symbols[0].symbol.kind, SymbolKind::Variable);
    assert_eq!(symbols[0].path, path);
}
