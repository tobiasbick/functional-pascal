namespace Fpas.Compiler.Tests.Syntax;

internal static class BoundaryCases
{
    internal static readonly (string Name, string Text)[] Gaps =
    [
        ("none", ""), ("space", " "), ("tab", "\t"), ("lf", "\n"),
        ("crlf", "\r\n"), ("cr", "\r"), ("mixed", " \t "),
        ("nbsp", "\u00A0"), ("em-space", "\u2003"),
        ("comment-lf", "// boundary\n"), ("comment-crlf", "// boundary\r\n"), ("comment-cr", "// boundary\r"),
        ("standalone-comment", "\n// boundary\n"), ("detached-comment", "\n// boundary\n\n")
    ];

    internal static readonly (string Name, string Atoms)[] Programs =
    [
        ("bindings-formals-named-calls", "program|Boundary|;|function|Add|(|First|:|int|;|Second|:|int|)|:|int|;|begin|return|First|+|Second|;|end|function|;|begin|const|Value|:|int|:=|Add|(|First|:=|1|,|Second|:=|2|)|;|end|;|end|program|;"),
        ("qualified-members-arrays-indexes", "program|Boundary|;|uses|System|.|Console|as|Console|;|begin|var|Values|:|array|of|int|:=|[|1|,|2|]|;|Values|[|0|]|:=|3|;|Console|.|WriteLine|(|Values|[|0|]|)|;|end|;|end|program|;"),
        ("generic-declarations-and-nesting", "program|Boundary|;|type|Box|<|T|>|=|record|public|Value|:|T|;|end|record|;|type|Holder|<|T|>|=|record|public|Value|:|Box|<|Box|<|T|>|>|;|end|record|;|begin|const|Value|:|Holder|<|int|>|:=|new|Holder|<|int|>|(|new|Box|<|Box|<|int|>|>|(|new|Box|<|int|>|(|42|)|)|)|;|end|;|end|program|;"),
        ("generic-unions-protocols-patterns", "program|Boundary|;|type|Choice|<|T|>|=|union|Empty|;|Value|(|Item|:|T|)|;|end|union|;|type|IValue|<|T|>|=|protocol|Value|:|T|;|end|protocol|;|begin|const|Item|:|Choice|<|int|>|:=|Choice|<|int|>|.|Value|(|1|)|;|case|Item|of|when|Choice|.|Value|(|const|Value|:|int|)|:|discard|Value|;|when|Choice|.|Empty|:|discard|0|;|end|case|;|end|;|end|program|;"),
        ("generic-constraints", "unit|Boundary|;|public|type|Box|<|T|>|=|where|T|:|class|;|record|public|Value|:|T|;|end|record|;|end|unit|;"),
        ("generic-result-and-arrow", "program|Boundary|;|type|Box|<|T|>|=|record|public|Value|:|T|;|end|record|;|begin|const|Make|:|function|(|)|:|Box|<|Box|<|int|>|>|:=|function|(|)|:|Box|<|Box|<|int|>|>|=>|new|Box|<|Box|<|int|>|>|(|new|Box|<|int|>|(|42|)|)|;|discard|Make|(|)|;|end|;|end|program|;"),
        ("nullable-suffix-and-coalescing", "program|Boundary|;|begin|const|Maybe|:|int|?|:=|null|;|const|Value|:|int|:=|Maybe|??|42|;|end|;|end|program|;"),
        ("comparisons-and-prefix-signs", "program|Boundary|;|begin|const|Value|:|int|:=|1|-|-|2|;|const|Smaller|:|bool|:=|not|(|Value|>=|3|)|;|if|Smaller|or|Value|=|3|then|discard|Value|;|end|if|;|end|;|end|program|;"),
        ("arithmetic-punctuation", "program|Boundary|;|begin|const|Quotient|:|double|:=|6.0D|/|-|2.0D|;|const|Product|:|int|:=|3|*|+|2|;|const|Total|:|int|:=|Product|+|+|2|;|const|Remainder|:|int|:=|10|mod|3|;|const|Shifted|:|int|:=|1|shl|2|;|end|;|end|program|;"),
        ("numeric-member-dots", "program|Boundary|;|begin|const|Whole|:|string|:=|42|.|ToString|(|)|;|const|Real|:|string|:=|1.5D|.|ToString|(|)|;|end|;|end|program|;"),
        ("literal-spellings", "program|Boundary|;|begin|const|Number|:|ulong|:=|$F_FUL|;|const|Scale|:|double|:=|1.5e-3D|;|const|Letter|:|char|:=|#65|;|const|Text|:|string|:=|'it''s // text'|;|end|;|end|program|;"),
        ("numeric-ranges-and-case-expression", "program|Boundary|;|type|Small|=|int|range|1|..|3|;|begin|const|Value|:|Small|:=|new|Small|(|2|)|;|discard|case|Value|of|when|1|..|2|:|true|;|else|false|;|end|case|;|end|;|end|program|;"),
        ("word-operators-and-control-endings", "program|Boundary|;|begin|for|Index|:|int|:=|1|to|2|do|discard|Index|div|1|;|end|for|;|var|Count|:|int|:=|0|;|while|Count|<|1|do|Count|:=|Count|+|1|;|end|while|;|repeat|Count|:=|Count|-|1|;|until|Count|=|0|;|end|;|end|program|;"),
        ("escaped-identifiers", "program|Boundary|;|function|@event|(|@in|:|int|)|:|int|;|begin|return|@in|;|end|function|;|begin|const|@out|:|int|:=|@event|(|1|)|;|end|;|end|program|;")
    ];

    internal static readonly (string Name, string Prefix, string Atom, string Suffix)[] Atomic =
    [
        ("assignment", "program Boundary;begin const Value:int", ":=", "1;end;end program;"),
        ("less-equal", "program Boundary;begin discard 2", "<=", "3;end;end program;"),
        ("greater-equal", "program Boundary;begin discard 2", ">=", "1;end;end program;"),
        ("not-equal", "program Boundary;begin discard 2", "<>", "3;end;end program;"),
        ("arrow", "program Boundary;begin const F:function(int):int:=function(Value:int):int", "=>", "Value+1;end;end program;"),
        ("coalescing", "program Boundary;begin const Maybe:int?:=null;discard Maybe", "??", "42;end;end program;"),
        ("range", "program Boundary;type Small=int range 1", "..", "3;begin end;end program;"),
        ("comment-start", "program Boundary;begin const Value:int:=1", "//", " ⛄\n;end;end program;"),
        ("quote-escape", "program Boundary;begin const Text:string:='it", "''", "s';end;end program;"),
        ("decimal-separators", "program Boundary;begin const Value:int:=", "1_000", ";end;end program;"),
        ("hexadecimal", "program Boundary;begin const Value:int:=", "$F_F", ";end;end program;"),
        ("numeric-suffix", "program Boundary;begin const Value:ulong:=", "42UL", ";end;end program;"),
        ("real-exponent-suffix", "program Boundary;begin const Value:double:=", "1.5e-3D", ";end;end program;"),
        ("character-code", "program Boundary;begin const Value:char:=", "#65", ";end;end program;"),
        ("escaped-name", "program Boundary;begin const ", "@event", ":int:=1;end;end program;"),
        ("keyword", "", "program", " Boundary;begin end;end program;")
    ];
}
