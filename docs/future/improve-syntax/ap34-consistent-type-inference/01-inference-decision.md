# AP34.1: Inference decision

Package: [AP34](README.md). Status: complete; decision work package.

## Selected binding rule

Local bindings in statement lists may omit their annotation whenever the
initializer has a fully determined type. The expression form does not control
eligibility: literals, ordinary calls, factories, operators, conversions,
field/index access, closures with complete signatures and conditional expressions
use the same rule. The inferred type is fixed for the binding's lifetime.
Explicit annotations remain valid and provide expected types where needed.

Parameters, routine results and record fields remain explicitly typed.
Declarations outside statement lists, including program/unit declarations and
routine declarations before `begin`, also retain required annotations.

## Selected generic argument rule

1. Collect type evidence from supplied arguments, constructor fields/payloads
   and receivers using their declared signatures. Named argument order does not
   change the inferred arguments.
2. Fill only unresolved parameters from the expected type supplied by an
   annotation, assignment, return, argument, field or callable signature.
3. Reject conflicting evidence, validate constraints and reject parameters that
   remain unresolved. Expected types never replace an already determined type.

Direct routine/member calls, constructors and generic callable values use this
same rule. Imported signatures provide the same evidence as local signatures.
Defaults do not infer generic arguments. No common numeric type, implicit value
conversion or default concrete type is selected; AP33's numeric contract applies.

## Predicted behavior

These are AP34.2 target rules, rather than claims of current compiler support.
Examples are local statements with their referenced routines/types declared.

| Case | Selected result |
| --- | --- |
| `const X := ReadValue();` where ReadValue returns integer | Infer integer, just as `[ReadValue()]` infers array of integer. |
| Constructor moved behind a non-generic factory | Its known result remains eligible for local inference. |
| `const X := 1 + 2;` or a field/index access with a known type | Infer the expression's checked result type. |
| `const X := Empty();` for `function Empty<T>(): Option of T` | Reject: T has no evidence or expected type. |
| `const X: Option of integer := Empty();` | Expected result fills T with integer. |
| Empty assigned to `function(): Option of integer`, then invoked | Same T as the annotated direct call. |
| `const X: Option of real := Identity(Some(1));` for generic Identity | Reject: arguments determine integer; the expected real type cannot override it. |
| `const X := [];`, `const X := None;`, or an unresolved generic payloadless variant | Require a complete expected type; do not guess. |
| `const X: array of integer := [];` or `const X: Option of integer := None;` | Annotation supplies the missing type. |
| `[1, 2.0]`, even in an annotated real array | Reject; convert integer explicitly with IntToReal or use matching literal types. |
| Generic field defaults | Checked against the declaration, without supplying inference evidence. |

## Relationship to Q14 and alternatives

AP22 remains complete for its delivered contract. AP34.2 replaces only Q14's
restriction to selected initializer forms in statement-local bindings. Its
explicit signature/declaration boundaries and rejection of unknown types remain.

Mandatory annotations on every binding would remove useful existing inference
and require broad consumer migration. Retaining Q14 would preserve the
call-versus-collection and constructor-versus-factory differences. The selected
rule removes those differences while retaining explicit interface types.

## Migration and delivery boundary

Existing valid annotations can remain; this decision requires no general
annotation-removal sweep. AP34.2 changes inference and outdated rejection
fixtures, updates diagnostic repairs, and aligns local/generic documentation
and language-service type display. It preserves existing evaluation order,
constraints and independently compiled unit behavior.

The recorded decision completes AP34.1. This update changes planning files
only; AP34.2 has not been implemented.

## Documentation and verification

The current local/generic handbook remains unchanged until AP34.2. Its
implementation must verify the table through normal checking and execution,
including direct/callable/member contexts, named arguments, imported signatures,
conflicts and unresolved types. Rejections identify the missing or conflicting
type and show a concrete annotation or explicit conversion that repairs it.

## Open decisions

None within this work package's selected scope.
