//! Long expressions expand automatically without source-authored line breaks.

mod common;

#[test]
fn long_builder_calls_expand_each_argument_at_the_structural_indent() {
    common::assert_golden(
        "automatic builder layout",
        "program T; begin return Overlay('Quit', [Tui.TuiElementBuilders.MakeButtonWithMnemonic(Tui.TuiControlId.Create(Actions.DiscardAndQuitControl), 'Discard', 'D', Tui.TuiAction.Create(Actions.DiscardAndQuitAction))]); end.",
        "program T;\n\nbegin\n  return Overlay(\n    'Quit',\n    [\n      Tui.TuiElementBuilders.MakeButtonWithMnemonic(\n        Tui.TuiControlId.Create(Actions.DiscardAndQuitControl),\n        'Discard',\n        'D',\n        Tui.TuiAction.Create(Actions.DiscardAndQuitAction)\n      )\n    ]\n  );\nend.\n",
    );
}

#[test]
fn automatic_call_wrapping_observes_the_100_column_boundary_and_unicode_width() {
    for text in [
        "x".repeat(90),
        format!("{}界", "x".repeat(88)),
        format!("{}é", "x".repeat(89)),
    ] {
        let source = format!("program T; begin Foo('{text}'); end.");
        let expected = format!("program T;\n\nbegin\n  Foo('{text}');\nend.\n");
        common::assert_golden("100 columns", &source, &expected);
        let source = format!("program T; begin Foo('{text}x'); end.");
        let expected = format!("program T;\n\nbegin\n  Foo(\n    '{text}x'\n  );\nend.\n");
        common::assert_golden("101 columns", &source, &expected);
    }
}

#[test]
fn authored_argument_groups_keep_their_breaks_and_wrap_overlong_groups() {
    common::assert_golden(
        "authored groups and automatic width",
        "program T; begin Foo(\nShortA, ShortB,\nVeryLongArgumentAlphaWithAdditionalCharacters, VeryLongArgumentBetaWithAdditionalCharacters, VeryLongArgumentGammaWithAdditionalCharacters\n); end.",
        "program T;\n\nbegin\n  Foo(\n    ShortA, ShortB,\n    VeryLongArgumentAlphaWithAdditionalCharacters, VeryLongArgumentBetaWithAdditionalCharacters,\n    VeryLongArgumentGammaWithAdditionalCharacters\n  );\nend.\n",
    );
    common::assert_golden(
        "authored title and collection group",
        "program T; begin return Overlay(\n'Quit', [\nTui.TuiElementBuilders.MakeButtonWithMnemonic(Tui.TuiControlId.Create(Actions.DiscardAndQuitControl), 'Discard', 'D', Tui.TuiAction.Create(Actions.DiscardAndQuitAction))\n]\n); end.",
        "program T;\n\nbegin\n  return Overlay(\n    'Quit', [\n      Tui.TuiElementBuilders.MakeButtonWithMnemonic(\n        Tui.TuiControlId.Create(Actions.DiscardAndQuitControl),\n        'Discard',\n        'D',\n        Tui.TuiAction.Create(Actions.DiscardAndQuitAction)\n      )\n    ]\n  );\nend.\n",
    );
}

#[test]
fn binary_operands_wrap_before_a_nested_call_needs_expansion() {
    common::assert_golden(
        "binary wrapping before nested call",
        "program T; begin return 'a moderately long prefix with some extra text' + FormatValue(VeryLongArgumentAlpha, VeryLongArgumentBeta); end.",
        "program T;\n\nbegin\n  return 'a moderately long prefix with some extra text' +\n    FormatValue(VeryLongArgumentAlpha, VeryLongArgumentBeta);\nend.\n",
    );
}
