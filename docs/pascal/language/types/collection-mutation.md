# Collection mutation

Native array, dictionary and string operations without InPlace do not directly
mutate their receiver. Queries return answers; transformations return new values.
Mutating counterparts require InPlace. There are no Sorted/Reversed aliases,
free native aliases or optional receiver markers.

```pascal
const Original := [3, 1, 2];
const Sorted := Original.Sort();
var Working := Original;
Working.SortInPlace();
Working.PushInPlace(4);
// Original remains [3, 1, 2].
```

## Writable storage and results

InPlace receivers must be writable variables, writable record fields, array
elements, forwarded var parameters or captured writable bindings. Const roots,
read-only parameters/Self/loop variables, dictionary entries, literals, grouped
and computed values are rejected (FP3028). Use the value operation and consume its
result when a receiver is not writable. Explicit arguments remain read-only and
cannot be marked var (FP3027).

InPlace transformations preserve static receiver type and return no value, so
they can only finish a statement chain. PopInPlace returns the removed element.
MapInPlace/FlatMapInPlace retain the exact element/value type; use Map/FlatMap for
type changes. Queries and conversions such as Join, Split, Keys, Values and
general Reduce have no InPlace counterparts.

## Evaluation, callbacks and failure

Receiver root and indices are selected once before explicit arguments. Arguments
run once in written order, including named calls. InPlace reads selected storage
after argument evaluation, making earlier argument writes visible. It computes
the transformed value and commits only after success. Bounds/path checks still
apply when accessing storage.

Callback operations visit a snapshot taken after explicit arguments finish.
Callback writes do not change visited elements. On success the computed result
replaces the selected receiver, including callback writes to that storage. On
failure no computed result is committed; completed argument/callback side effects
remain visible. Early try exits retain completed writes. This is not a transaction
over callback effects, and value operations are not necessarily pure.

Value copies remain independent. InPlace describes caller storage updates, not
an allocation-free guarantee; copy-on-write and runtime algorithms may allocate.
String InPlace calls replace the variable; character slots remain read-only.

## User routines and tasks

User mutation uses ordinary explicit var parameters, including Name := var Storage
in named calls. Record Self remains read-only; bound methods capture values. An
InPlace suffix on a user routine does not grant writable Self. Exact-type, alias
and reference lifetime rules remain in force.

InPlace receivers cannot cross go boundaries (FP3030). Value calls, including
Push and Pop, may run in tasks; receivers and explicit arguments are evaluated
in the caller before spawning.

## See also

- [Array operations](array/README.md)
- [Dictionary operations](dictionary-operations.md)
- [String operations](string/README.md)
- [Var parameters](../functions/var-parameters.md)
