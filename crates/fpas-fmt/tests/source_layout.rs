//! Source layout is an author choice; formatting normalizes its indentation.

use crate::common;

#[test]
fn builder_layout_preserves_concatenation_groups_and_nested_arguments() {
    common::assert_golden(
        "builder layout",
        "program T; begin return Builders.MakeRule(\n' NOTES  ·  ' +\nIntToStr(Filtered(State.Notes, '').Length()) +\n' active' + Issues + Dirty +\n'  ·  Ctrl+K commands '\n); end.",
        "program T;\n\nbegin\n  return Builders.MakeRule(\n    ' NOTES  ·  ' +\n    IntToStr(Filtered(State.Notes, '').Length()) +\n    ' active' + Issues + Dirty +\n    '  ·  Ctrl+K commands '\n  );\nend.\n",
    );
    common::assert_golden(
        "nested collection builders",
        "program T; begin return Builders.Status([\nHint(State.Status),\nButton(\nId(1),\n'New note',\nAction(2)\n),\nKey('F1', 'help')\n]); end.",
        "program T;\n\nbegin\n  return Builders.Status([\n    Hint(State.Status),\n    Button(\n      Id(1),\n      'New note',\n      Action(2)\n    ),\n    Key('F1', 'help')\n  ]);\nend.\n",
    );
}

#[test]
fn indivisible_long_literals_are_preserved() {
    let text = "x".repeat(120);
    common::assert_golden(
        "indivisible long literal",
        &format!("program T; begin return '{text}'; end."),
        &format!("program T;\n\nbegin\n  return '{text}';\nend.\n"),
    );
}

#[test]
fn grouped_arguments_and_statement_sections_survive_normalization() {
    common::assert_golden(
        "groups and sections",
        "program T; begin\nFoo(\na,b,\nc,d\n);\n\n\nBar();\nBaz();\nend.",
        "program T;\n\nbegin\n  Foo(\n    a, b,\n    c, d\n  );\n\n  Bar();\n  Baz();\nend.\n",
    );
}

#[test]
fn list_comments_stay_with_their_arguments_and_closing_delimiter() {
    common::assert_golden(
        "list comments",
        "program T; begin Foo(\n// first argument\nA, // keep with A\n// second argument\nB\n// before close\n); end.",
        "program T;\n\nbegin\n  Foo(\n    // first argument\n    A, // keep with A\n    // second argument\n    B\n    // before close\n  );\nend.\n",
    );
}

#[test]
fn every_expression_list_uses_the_same_structural_layout() {
    for (name, source, expected) in [
        (
            "named arguments",
            "Foo(\nLeft := 1,\nRight := 2\n);",
            "  Foo(\n    Left := 1,\n    Right := 2\n  );",
        ),
        (
            "postfix method",
            "return Factory.Create().Build(\nA,\nB\n).Value;",
            "  return Factory.Create().Build(\n    A,\n    B\n  ).Value;",
        ),
        (
            "discarded postfix breaks",
            "return Format(Factory.Create()\n.Length());",
            "  return Format(Factory.Create().Length());",
        ),
        (
            "constructor",
            "return Some(\n1 +\n2\n);",
            "  return Some(\n    1 +\n    2\n  );",
        ),
        (
            "compact record update",
            "Foo(P with X := 1; end with);",
            "  Foo(P with X := 1; end with);",
        ),
        (
            "standalone binary nesting",
            "return A +\nB *\nC +\nD;",
            "  return A +\n    B *\n    C +\n    D;",
        ),
        (
            "index",
            "return Items[\nIndex +\n1\n];",
            "  return Items[\n    Index +\n    1\n  ];",
        ),
        (
            "dictionary",
            "return ['one': 1,\n 'two': 2];",
            "  return [\n    'one': 1,\n    'two': 2\n  ];",
        ),
        (
            "separate collection call ending",
            "Foo([\n1, 2\n]\n);",
            "  Foo(\n    [\n      1, 2\n    ]\n  );",
        ),
        (
            "multiline string token",
            "Foo('a\nb');",
            "  Foo('a'#10'b');",
        ),
        (
            "empty commented call",
            "Foo(\n// placeholder\n);",
            "  Foo(\n    // placeholder\n  );",
        ),
        (
            "empty commented dictionary",
            "return [:\n// placeholder\n];",
            "  return [:\n    // placeholder\n  ];",
        ),
    ] {
        common::assert_golden(
            name,
            &format!("program T; begin {source} end."),
            &format!("program T;\n\nbegin\n{expected}\nend.\n"),
        );
    }
}

#[test]
fn nested_formal_parameters_use_the_owning_declarations_indentation() {
    common::assert_golden(
        "nested formal parameters",
        "unit T; type Holder = record function Apply(\nSelf: Holder; A: integer;\nCallback: function(\nX: integer;\nY: integer\n): integer\n): integer; begin return A; end function; end record; end unit;",
        "unit T;\n\ntype Holder = record\n  function Apply(\n    Self: Holder; A: integer;\n    Callback: function(\n      X: integer;\n      Y: integer\n    ): integer\n  ): integer;\n  begin\n    return A;\n  end function;\nend record;\nend unit;\n",
    );
}

#[test]
fn binary_comments_remain_at_their_operand_boundary() {
    common::assert_golden(
        "binary operand comments",
        "program T; begin return Foo(\n'a' + // first group\n// next part\n'b' + 'c'\n); end.",
        "program T;\n\nbegin\n  return Foo(\n    'a' + // first group\n    // next part\n    'b' + 'c'\n  );\nend.\n",
    );
}

#[test]
fn logical_source_lines_work_for_lf_crlf_and_bare_cr() {
    let source = "program T;\nbegin\nreturn Foo(\n'界' +\n'é'\n);\n\nBar();\nend.\n";
    let expected =
        "program T;\n\nbegin\n  return Foo(\n    '界' +\n    'é'\n  );\n\n  Bar();\nend.\n";
    for ending in ["\n", "\r\n", "\r"] {
        common::assert_golden(
            "logical source lines",
            &source.replace('\n', ending),
            expected,
        );
    }
}

#[test]
fn multiline_statement_conditions_put_then_on_the_clause_indent() {
    common::assert_golden(
        "then after multiline if and elsif conditions",
        "program T; begin if First(\nA,\nB\n) then null; elsif Second(\nC,\nD\n) then // branch body\nnull; elsif Ready then null; end if; end.",
        "program T;\n\nbegin\n  if First(\n    A,\n    B\n  )\n  then\n    null;\n  elsif Second(\n    C,\n    D\n  )\n  then\n    // branch body\n    null;\n  elsif Ready then\n    null;\n  end if;\nend.\n",
    );
    common::assert_golden(
        "then after an automatically wrapped condition",
        "program T; begin if CheckAllConditions(VeryLongConditionArgumentAlpha, VeryLongConditionArgumentBeta, VeryLongConditionArgumentGamma) then null; end if; end.",
        "program T;\n\nbegin\n  if CheckAllConditions(\n    VeryLongConditionArgumentAlpha,\n    VeryLongConditionArgumentBeta,\n    VeryLongConditionArgumentGamma\n  )\n  then\n    null;\n  end if;\nend.\n",
    );
    common::assert_golden(
        "condition ending comment without an extra blank line",
        "program T; begin if case 1 of when 1: true; else false; end case // condition result\nthen null; end if; end.",
        "program T;\n\nbegin\n  if case 1 of\n    when 1: true;\n    else false;\n  end case // condition result\n  then\n    null;\n  end if;\nend.\n",
    );
}

#[test]
fn wrapped_logical_operators_occupy_their_own_continuation_line() {
    for separator in [" ", "\n"] {
        common::assert_golden(
            "Notes condition groups with automatic or authored wrapping",
            &format!(
                "program T; begin if ((State.Selected >= 0) and (State.Selected < State.Notes.Length())) and{separator}((State.Focus >= Actions.TitleControl) and (State.Focus <= Actions.BodyControl)) then begin return State.Focus; end; end if; end."
            ),
            "program T;\n\nbegin\n  if ((State.Selected >= 0) and (State.Selected < State.Notes.Length()))\n    and\n    ((State.Focus >= Actions.TitleControl) and (State.Focus <= Actions.BodyControl))\n  then\n    begin\n      return State.Focus;\n    end;\n  end if;\nend.\n",
        );
    }
    for op in ["and", "or", "xor"] {
        common::assert_golden(
            "logical source breaks, comments and compact branches",
            &format!(
                "program T; begin if A {op}\nB {op} // connector\n// next group\nC then null; elsif (D\n{op}\nE) then null; elsif F {op} G then null; end if; end."
            ),
            &format!(
                "program T;\n\nbegin\n  if A\n    {op}\n    B\n    {op} // connector\n    // next group\n    C\n  then\n    null;\n  elsif (\n    D\n    {op}\n    E\n  )\n  then\n    null;\n  elsif F {op} G then\n    null;\n  end if;\nend.\n"
            ),
        );
    }
    common::assert_golden(
        "logical connector after a multiline operand ending comment",
        "program T; begin if case 1 of when 1: true; else false; end case // left group\nand Ready then null; end if; end.",
        "program T;\n\nbegin\n  if case 1 of\n    when 1: true;\n    else false;\n  end case // left group\n    and\n    Ready\n  then\n    null;\n  end if;\nend.\n",
    );
}
