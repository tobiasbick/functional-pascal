# Passive debugger expressions

The own debugger evaluates a bounded FPAS subset against one current stopped
frame. Batch and DAP use the same core operation and return independent value or
error results. Expressions never invoke target methods, getters, conversions,
operators, constructors, display attributes or proxies. They do not mutate
target storage. The [core reference](debugger-core.md) owns stop/value lifetimes.

## Names and stored access

Names are case-sensitive. Active lexical locals, arguments and source captures
take precedence over enclosing static scopes. Recognized FPAS instance receivers
use `self`. Source scopes follow the original async kickoff or closure owner;
they do not substitute the state machine's bookkeeping for source globals.

Member access reads source-mapped record/variant/domain fields or ordinary stored
CLR fields. A domain's Value uses the decoded primitive source view. Custom
property getters report `requires_execution`. Unresolved fields have no fallback
through a managed evaluator. Raw compiler backing names remain an inspection
view rather than another expression syntax.

Qualified static access uses an actual loaded CLR type name or primitive alias,
such as `int.MaxValue`. Own lexical scopes can access their private fields;
qualified foreign fields must be public. Import aliases are not expanded.
Ambiguous loaded type names are errors. Static reads retain the concrete loaded
module and generic source scope, pass the selected frame for thread-relative
storage and do not initialize a type.

Array indexes use the actual rank, lengths and lower bounds. Their values must
convert implicitly to int. String indexes select UTF-16 code units, preserving
lone surrogate data. A string index outside retained content reports truncation
when the complete runtime string is longer; it does not invent a character.

## Primitive operations

Literals use FPAS spelling, including multiline apostrophe strings, doubled apostrophes,
decimal character codes, hexadecimal integers, digit separators and numeric
suffixes. Unsuffixed integers fit int; negative minimum literals retain their
selected signed type. Finite real literals and runtime NaN/infinity values keep
their separate language rules. Escaped identifiers and line comments follow
FPAS spelling. Physical line endings in a string evaluate as LF.

Supported operators, from highest to lowest precedence:

| Operators | Passive behavior |
| --- | --- |
| Member/index access and parentheses | Stored fields and array/string elements. |
| Unary `+`, `-`, `not` | Primitive numeric promotion, Boolean negation and integral complement. |
| `*`, `/`, `div`, `mod` | `/` converts two integral operands to double. Use div/mod for integral quotient and remainder. |
| `+`, `-` | Primitive arithmetic; string/char concatenation. |
| `shl`, `shr` | CLR promotion and masked shift counts. |
| `<`, `<=`, `>`, `>=` | Primitive numeric ordering. |
| `=`, `<>` | Primitive equality and null tests for reference values. |
| `and`, `xor`, `or` | Boolean logic or integral bit operations. Boolean and/or short-circuit; xor is eager. |

Use one operator kind per logical chain. Parentheses can group a separate chain,
as in `(A and B) or C`.

Numeric promotion follows the compiler's CLR rules, including representable
integer constants with unsigned operands. Integral arithmetic checks overflow;
bit and shift results retain CLR widths. Float operands round to float before
the operation. Decimal does not implicitly mix with binary floating point.
String concatenation does not format arbitrary objects.

Short-circuit type/name checking reads metadata without calculating skipped
index values or arithmetic. Index types and rank remain checked in skipped
branches. Unavailable declared module/type information is an explicit error.
Stored-field signature inspection is bounded and does not guess a load context
for an unresolved external type reference.

Calls, construction, assignment, await, casts, unchecked expressions, null
coalescing, reference identity comparisons between non-null objects and nominal
enum/domain/nullable-value operators are outside this subset. Inspect an enum directly or use
a domain's passive Value when primitive arithmetic is needed. These limits do
not revise FPAS program syntax or introduce a second general language evaluator.

## Bounds and availability

| Bound | Limit |
| --- | --- |
| Expression text | 8192 UTF-16 code units. |
| Syntax tree | 4096 nodes, depth 64. |
| Parser nesting | 64 levels. |
| Evaluator/type traversal | 16384 operations. |
| Binding/value lookup entries | 65536 per expression. |
| Metadata type discovery | 65536 per expression. |
| Field type description | 16 levels, 128 nodes, 64 generic arguments. |
| Source static scopes | 64 enclosing types. |

The core's ordinary value-path, object, frame, string and message limits also
apply. Whole values can be returned with truncation markers; operations needing
omitted string content report `truncated_value`. Availability errors do not
change the current stop or revision. Continuing invalidates the frame and every
value reference; a later stop cannot reuse an earlier expression's storage.

## Verification

`ExpressionInspectionTests` uses actual stopped FPAS programs and independently
expected data. It checks promotion, exact extrema, overflow, decimal/float rules,
UTF-16, fields, arrays, lexical/static precedence, short-circuit validation,
syntax/resource limits and expired/fresh frames. Compound-value tests add
negative runtime array bounds and hostile getter/operator/display sentinels.
The required native Windows/WSL Debug/Release matrix is described in
[test instructions](../../tests/README.md#own-coreclr-connection-tests).
