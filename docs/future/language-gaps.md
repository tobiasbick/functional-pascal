# Language gaps and deferred extensions

Record confirmed omissions from the [baseline](../baseline-comparison.md) with
their owning rule and reproducer.
Record concrete maintenance blockers in task-specific documents under
`docs/future/`; current toolchain procedures belong in the
[development workflow](../build/development.md).

## SDK and build gaps

- [ ] Compile `.resx` files in FPAS projects and embed their resources in the
  emitted assembly. Support culture-specific satellite assemblies and verify
  that `ResourceManager` loads the selected culture and falls back to the
  neutral language in build and publish outputs. An `EmbeddedResource` item
  currently fails with `FP5001`.

## Scope boundary

Source event declarations, expression-tree conversion, user-authored class
hierarchies, custom awaiters, async iterators, unsafe/byref-like features,
operator declarations, automatic extension-method discovery, and interface
variance/default bodies remain outside the agreed scope. Additional platform/framework
coverage remains a separate toolchain proposal.
