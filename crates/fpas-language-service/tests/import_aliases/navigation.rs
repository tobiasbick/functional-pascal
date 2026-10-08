//! Navigation preserves canonical declarations through source aliases.

use super::{UNIT, fixture};

#[test]
fn definitions_hover_signatures_and_named_arguments_follow_aliases() {
    let source =
        "program App;\nuses Demo.Math as M;\nbegin\n  discard m.Answer(Value := 7);\nend.\n";
    let mut f = fixture(source);
    let answer_offset = source.find("Answer(").expect("call");
    let definitions = f
        .service
        .definitions(&f.main, answer_offset)
        .expect("definition")
        .value;
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].path, f.unit);
    assert_eq!(definitions[0].symbol.qualified_name, "Demo.Math.Answer");
    let hover = f
        .service
        .hover(&f.main, answer_offset)
        .expect("hover")
        .value
        .expect("callable hover");
    assert!(hover.contents.contains("function Answer(Value: integer)"));
    assert_eq!(
        hover.documentation.as_deref(),
        Some("Returns its argument.")
    );
    let signature = f
        .service
        .signature_help(&f.main, source.find("7);").expect("argument") + 1)
        .expect("signature")
        .value
        .expect("callable signature");
    assert_eq!(signature.active_parameter, Some(0));
    assert!(signature.signature.label.contains("Value: integer"));
    let parameter = f
        .service
        .definitions(&f.main, source.find("Value :=").expect("label"))
        .expect("parameter definition")
        .value;
    assert_eq!(parameter.len(), 1);
    assert_eq!(parameter[0].path, f.unit);
    assert_eq!(parameter[0].symbol.name, "Value");
}

#[test]
fn alias_has_a_local_definition_and_import_path_navigates_to_unit() {
    let source = "program App;\nuses Demo.Math as M;\nbegin\n  discard M.Answer(1);\nend.\n";
    let mut f = fixture(source);
    let alias = f
        .service
        .definitions(&f.main, source.find("M.Answer").expect("namespace"))
        .expect("alias definition")
        .value;
    assert_eq!(alias.len(), 1);
    assert_eq!(alias[0].path, f.main);
    assert_eq!(
        alias[0].symbol.selection_span.offset(),
        source.find("as M").expect("alias declaration") + 3
    );
    let unit = f
        .service
        .definitions(&f.main, source.find("Demo.Math").expect("import"))
        .expect("unit definition")
        .value;
    assert_eq!(unit.len(), 1);
    assert_eq!(unit[0].path, f.unit);
    assert!(
        f.service
            .definitions(&f.main, source.find("as M").expect("modifier"))
            .expect("contextual modifier")
            .value
            .is_empty()
    );
}

#[test]
fn references_and_symbol_rename_share_identity_across_different_aliases() {
    let source = "program App;\nuses Demo.Math as M;\nbegin\n  discard M.Answer(1);\nend.\n";
    let mut f = fixture(source);
    let other_source = "unit Demo.Client;\nuses Demo.Math as Other;\npublic function GetValue(): integer;\nbegin return Other.Answer(2); end function;\nend unit;\n";
    let other = f.temp.write("src/client.fpas", other_source);
    f.service = fpas_language_service::LanguageService::load(&f.temp.join("demo.fpasprj"));
    let offset = source.find("Answer(").expect("call");
    let references = f
        .service
        .references(&f.main, offset, true)
        .expect("references")
        .value;
    assert_eq!(references.len(), 3, "{references:?}");
    let edits = f
        .service
        .rename(&f.main, offset, "Compute")
        .expect("rename routine")
        .value;
    assert_eq!(edits.len(), 3);
    assert!(super::apply_edits(source, &edits, &f.main).contains("M.Compute(1)"));
    assert!(super::apply_edits(other_source, &edits, &other).contains("Other.Compute(2)"));
    assert!(super::apply_edits(UNIT, &edits, &f.unit).contains("function Compute("));
}

#[test]
fn original_path_short_names_private_symbols_and_private_members_stay_hidden() {
    let source = "program App;\nuses Demo.Math as M;\nbegin\n  discard Answer(1);\n  discard Demo.Math.Answer(1);\n  discard M.Secret();\n  discard M.Origin.Hidden;\nend.\n";
    let mut f = fixture(source);
    for needle in ["Answer(1)", "Math.Answer", "Secret", "Hidden"] {
        let mut offset = source.find(needle).expect("hidden reference");
        if needle == "Math.Answer" {
            offset += 5;
        }
        assert!(
            f.service
                .definitions(&f.main, offset)
                .expect("hidden definition")
                .value
                .is_empty(),
            "{needle}"
        );
    }
}

#[test]
fn named_types_use_import_environment_of_their_declaration() {
    let source = "program App;\nuses Demo.Math as M;\nbegin\n  const Local: M.Point := M.MakePoint();\n  discard Local.X;\nend.\n";
    let mut f = fixture(source);
    let types = f
        .service
        .type_definitions(&f.main, source.find("Local.X").expect("typed variable"))
        .expect("type definition")
        .value;
    assert_eq!(types.len(), 1);
    assert_eq!(types[0].path, f.unit);
    assert_eq!(types[0].symbol.qualified_name, "Demo.Math.Point");
    let make_type = f
        .service
        .type_definitions(&f.main, source.find("MakePoint").expect("function result"))
        .expect("result type")
        .value;
    assert_eq!(make_type.len(), 1);
    assert_eq!(make_type[0].symbol.qualified_name, "Demo.Math.Point");
    let field = f
        .service
        .definitions(&f.main, source.find("Local.X").expect("field") + 6)
        .expect("field definition")
        .value;
    assert_eq!(field.len(), 1);
    assert_eq!(field[0].symbol.qualified_name, "Demo.Math.Point.X");
}

#[test]
fn exported_result_types_resolve_aliases_in_the_defining_unit() {
    let source = "program App;\nuses Demo.Factory as F, Demo.Math as M;\ntype Point = record\n  Wrong: integer;\nend record;\nbegin\n  discard F.Make();\nend.\n";
    let mut f = fixture(source);
    let factory = f.temp.write("src/factory.fpas", "unit Demo.Factory;\nuses Demo.Math as Model;\npublic function Make(): Model.Point;\nbegin return Model.MakePoint(); end function;\nend unit;\n");
    f.service = fpas_language_service::LanguageService::load(&f.temp.join("demo.fpasprj"));
    let definitions = f
        .service
        .type_definitions(&f.main, source.find("Make()").expect("function result"))
        .expect("aliased result type")
        .value;
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].path, f.unit);
    assert_eq!(definitions[0].symbol.qualified_name, "Demo.Math.Point");
    let definition = f
        .service
        .definitions(&f.main, source.find("Make()").expect("call"))
        .expect("factory function")
        .value;
    assert_eq!(definition[0].path, factory);
}
