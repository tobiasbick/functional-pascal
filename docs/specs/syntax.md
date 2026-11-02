# Syntax grammar

This EBNF describes the baseline's parser shape. Semantic requirements, overload
resolution, definite assignment, coverage, and supported metadata signatures are
defined by the [language topics](../pascal/README.md). The
[implemented parser subset](../implementation/first-program.md) is smaller than
this full grammar; the grammar does not replace the semantic rules.

Notation: quoted words are terminals; `[...]` is optional, `{...}` repeats,
and `|` selects alternatives. `identifier` includes escaped `@identifier` names.
`literal`, `integral_literal`, and comments follow [basics](../pascal/language/basics.md).
Only `//` comments are recognized, ending at a line ending or EOF. Comments are
lexical trivia rather than statements. Preserve their text and source locations
for [declaration documentation](../pascal/language/comments.md); Markdown sections
do not add language tokens or change the grammar below.
Keywords are lowercase and case-sensitive. `>>` is two generic closers, not a
shift operator; shifts use `shr`/`shl`. `??` is one token for null coalescing.
`?` is used only in type syntax, including array element annotations and nullable
reference constraints; it is not a postfix expression operator.

## Sources and declarations

```ebnf
source          = program | unit ;
program         = ["async"], "program", qualified_name, ";", [imports],
                  {declaration}, "begin", statements, "end", ";",
                  "end", "program", ";" ;
unit            = "unit", qualified_name, ";", [imports],
                  {declaration}, "end", "unit", ";" ;
imports         = "uses", import, {",", import}, ";" ;
import          = qualified_name, "as", identifier ;
qualified_name  = identifier, {".", identifier} ;

declaration     = {attribute}, [visibility],
                  (type_declaration | function_declaration |
                   procedure_declaration | const_declaration | var_declaration) ;
visibility      = "public" | "private" ;
const_declaration = "const", identifier, ":", type, ":=", expression, ";" ;
var_declaration = "var", identifier, ":", type, ":=", expression, ";" ;

type_declaration = "type", identifier, [type_parameters], "=", {constraint},
                   (type | distinct_type | subrange_type |
                    record_body | union_body | enum_body), ";" ;
distinct_type   = "distinct", named_type ;
subrange_type   = named_type, "range", expression, "..", expression ;
type_parameters = "<", identifier, {",", identifier}, ">" ;
type_arguments  = "<", type, {",", type}, ">" ;
type            = named_type, {type_suffix} | callable_type ;
named_type      = identifier, [type_arguments],
                  {".", identifier, [type_arguments]} ;
type_suffix     = "?" | "[", {","}, "]" ;
callable_type   = "function", "(", [type_list], ")", ":", type
                | "procedure", "(", [type_list], ")" ;
type_list       = type, {",", type} ;
constraint      = "where", identifier, ":", constraint_item,
                  {",", constraint_item}, ";" ;
constraint_item = named_type | "class", ["?"] | "struct" | "new", "(", ")" ;

record_body     = "record", ["implements", named_type, {",", named_type}],
                  {record_member}, "end", "record" ;
record_member   = {attribute}, [visibility],
                  (identifier, ":", type, [":=", expression], ";" |
                   ["static"], (function_declaration | procedure_declaration)) ;
union_body      = "union", union_case, {union_case}, "end", "union" ;
union_case      = identifier, ["(", field_parameters, ")"], ";" ;
field_parameters = field_parameter, {";", field_parameter} ;
field_parameter = identifier, ":", type ;
enum_body       = "enum", [":", named_type], enum_case, {enum_case},
                  "end", "enum" ;
enum_case       = identifier, ["=", expression], ";" ;

function_declaration = ["async"], "function", identifier, [type_parameters],
                       "(", [parameters], ")", ":", type, ";",
                       {constraint}, contracts, "begin", statements,
                       "end", "function", ";" ;
procedure_declaration = "procedure", identifier, [type_parameters],
                        "(", [parameters], ")", ";", {constraint},
                        contracts, "begin", statements, "end", "procedure", ";" ;
contracts       = {requires_clause}, {ensures_clause} ;
requires_clause = "requires", expression, ";" ;
ensures_clause  = "ensures", ["(", identifier, ":", type, ")", ":"],
                  expression, ";" ;
parameters      = parameter, {";", parameter} ;
parameter       = ["ref" | "out" | "in" | "params"], identifier, ":", type,
                  [":=", expression] ;

attribute       = "[", named_type, ["(", [arguments], ")"], "]" ;
```

Empty unions/enums, nullable-byref types, invalid array ranks, unsupported generic
combinations, and misplaced attributes are rejected by semantic rules even if a
shape production accepts them. Attributes apply only to types/routines/record
fields in the baseline. Type annotations are required on value declarations at
every scope; no local-binding annotation is optional.
Programs terminate with `end program;`; units terminate with `end unit;`.
The program entry body's `begin` has its own `end;`. A separate `end program;`
then closes the program declaration. Both are required; a terminal period is invalid.
Distinct bases and subrange bases/bounds obey [data-domain rules](../pascal/language/data-domains.md).
They cannot have generic parameters or where constraints. A field default is a
runtime initializer, not necessarily a compile-time constant. Static is restricted
to record methods; it cannot modify a field or a local/free routine. A private
modifier on a top-level declaration restates its default unit-private visibility.
Public/private modifiers are not available on local declarations.
Ensures result bindings also require a type. A parenthesized Boolean expression
without the typed-binding-and-colon prefix is an ordinary result-free condition.
Contracts are ordered requires clauses followed by ensures clauses; the rules in
[contracts](../pascal/language/contracts.md) define allowed expressions and exits.

## Statements

```ebnf
statements      = {statement} ;
statement       = const_declaration | var_declaration
                | function_declaration | procedure_declaration
                | "begin", statements, "end", ";"
                | expression, ":=", expression, ";"
                | expression, ";"
                | "discard", expression, ";"
                | "return", [expression], ";"
                | "raise", [expression], ";"
                | "break", ";" | "continue", ";"
                | if_statement | case_statement | for_statement
                | while_statement | repeat_statement | try_statement
                | using_declaration | subscription ;

if_statement    = "if", expression, "then", statements,
                  {"elsif", expression, "then", statements},
                  ["else", statements], "end", "if", ";" ;
case_statement  = "case", expression, "of",
                  {"when", patterns, ["if", expression], ":", statements},
                  ["else", statements], "end", "case", ";" ;
for_statement   = "for", identifier, ":", type,
                  ( ":=", expression, ("to" | "downto"), expression
                  | "in", expression ),
                  "do", statements, "end", "for", ";" ;
while_statement = "while", expression, "do", statements, "end", "while", ";" ;
repeat_statement = "repeat", statements, "until", expression, ";" ;
try_statement   = "try", statements, {catch_clause},
                  ["finally", statements], "end", "try", ";" ;
catch_clause    = "catch", [identifier, ":", type],
                  ["when", expression], "do", statements ;
using_declaration = ["await"], "using", const_declaration ;
subscription    = ("subscribe" | "unsubscribe"), member_expression,
                  "with", expression, ";" ;
```

An expression statement must be a void call or void await. Assignment requires
writable storage. Type declarations are top-level only. A try statement needs at
least a catch or finally. Subscription's member expression must resolve to an
event; it excludes a record-update suffix so `with` is unambiguous here.
Using declarations require `const` and its explicit type. They are local
statements, not block openers; cleanup occurs on leaving the enclosing lexical
scope. `await using const` additionally requires an async body. See
[resource rules](../pascal/language/errors-and-resources.md) for scope, lifetime,
and disposal requirements.

## Expressions and patterns

```ebnf
expression      = coalesce_expression ;
coalesce_expression = or_expression, ["??", coalesce_expression] ;
or_expression   = xor_expression, {"or", xor_expression} ;
xor_expression  = and_expression, {"xor", and_expression} ;
and_expression  = equality_expression, {"and", equality_expression} ;
equality_expression = relation_expression,
                      [("=" | "<>"), relation_expression] ;
relation_expression = shift_expression,
                      [ ("<" | "<=" | ">" | ">="), shift_expression
                      | "is", type ] ;
shift_expression = additive_expression, {("shl" | "shr"), additive_expression} ;
additive_expression = product_expression, {("+" | "-"), product_expression} ;
product_expression = unary_expression,
                     {("*" | "/" | "div" | "mod"), unary_expression} ;
unary_expression = ("+" | "-" | "not" | "await"), unary_expression
                 | postfix_expression ;
postfix_expression = member_expression, [record_update] ;
member_expression = primary, {postfix_suffix} ;
postfix_suffix  = ".", identifier, [type_arguments]
                | "(", [arguments], ")"
                | "[", expression, {",", expression}, "]" ;
record_update   = "with", identifier, ":=", expression, ";",
                  {identifier, ":=", expression, ";"}, "end", "with" ;
primary         = literal | identifier, [type_arguments]
                | "(", expression, ")"
                | "[", [expression, {",", expression}], "]"
                | "new", named_type, ["?"],
                  ("(", [arguments], ")" | "[", expression, {",", expression}, "]")
                | "cast", "<", type, ">", "(", expression, ")"
                | "typeof", "(", type, ")"
                | "unchecked", "(", expression, ")"
                | if_expression | case_expression | lambda ;
arguments       = argument, {",", argument} ;
argument        = [identifier, ":="], ["ref" | "out" | "in"], expression ;
if_expression   = "if", expression, "then", expression,
                  {"elsif", expression, "then", expression},
                  "else", expression, "end", "if" ;
case_expression = "case", expression, "of",
                  {"when", patterns, ["if", expression], ":", expression, ";"},
                  ["else", expression, ";"], "end", "case" ;
lambda          = ["async"], "function", "(", [field_parameters], ")", ":", type,
                  ("=>", expression |
                   contracts, "begin", statements, "end", "function")
                | "procedure", "(", [field_parameters], ")",
                  contracts, "begin", statements, "end", "procedure" ;

patterns        = pattern, {",", pattern} ;
pattern         = "_" | "const", identifier, ":", type
                | "is", type, "as", identifier
                | constant_pattern, ["..", constant_pattern]
                | qualified_name, "(", [pattern_arguments], ")" ;
pattern_arguments = pattern_argument, {",", pattern_argument} ;
pattern_argument = [identifier, ":="], pattern ;
constant_pattern = ["-"], (literal | qualified_name) ;
```

`literal` includes strings, character codes, numeric forms, true, false, and null.
Semantic analysis restricts subrange bounds, pattern ranges, enum initializers,
attribute constants, and optional-parameter defaults to legal constant types.
An ordinary alias remains distinct from the `BaseType range Low..High` production.
The optional `?` after a new-expression's named type is allowed only for an
array allocation's element annotation, not for constructing a nullable class.

Disambiguate `<...>` type arguments from comparisons using standard syntactic
lookahead: a complete type-argument list in a valid name/postfix context is
required. In particular, comparison operands must not be consumed speculatively
as type arguments when the following token cannot continue that name. Diagnostics
must report ambiguous/invalid syntax rather than use symbol lookup to invent a
different parse. An explicit parenthesized comparison is always available.

An implementation must additionally validate clause pairing (`end function`
versus `end procedure`), generic arity, valid statement expressions, binding scope,
attribute placement, definite assignment, supported async results, and coverage.
Contract conditions must be bool, obey their expression restrictions, and use
correctly typed result bindings. Procedures/non-generic async Task functions
cannot bind a result; async Task<T> functions bind T rather than Task<T>.
