# Std.Tui2 handles and ownership

## Handle representation

Every live handle is application-scoped and generational. Its logical capability contains:

```text
application id
registry slot
generation
kind
```

Public handles are immutable records. FPAS does not currently provide opaque record fields, so callers can technically construct record literals. Such values have no authority until the registry validates all capability fields.

Every live-object constructor receives its owning `TuiApplication` explicitly. No constructor relies on ambient current-application state. Initial bounds come from an attached layout or a later explicit `TuiView.SetBounds` call.

Reused registry slots always receive a new generation. A stale handle can therefore never refer to a newly allocated object accidentally.

Implemented: unattached `TuiView` handles use this registry model and expose a live `Tag` property.
`TuiContainer` owns direct children through `Add` and destroys them through `Remove`. Nested
subtree ownership remains the next ownership step. `TuiDesktop.Create(App)` creates one explicit
root container for a live application.

## Validation

Every public operation validates:

- the application is live;
- the slot exists;
- the generation matches;
- the handle belongs to the expected application;
- the kind is accepted by the operation;
- the operation is legal in the current lifecycle phase.

Failure is a programming error with a concrete runtime diagnostic naming the operation and handle problem.

## Typed handles

Typed handles do not use inheritance or implicit structural conversion. Each control provides an explicit conversion, for example:

```pascal
TuiButton.AsView(Button: TuiButton): TuiView
TuiDialog.AsView(Dialog: TuiDialog): TuiView
TuiDialog.AsContainer(Dialog: TuiDialog): TuiContainer
TuiVerticalLayout.AsLayout(Layout: TuiVerticalLayout): TuiLayout
```

Containers accept `TuiView`; container operations accept `TuiContainer`; layout operations accept `TuiLayout`. Downcasts are not part of the public API.

## Ownership

- `TuiApplication` owns the registry, desktop, actions, and unattached live objects.
- A container owns each attached child subtree.
- A container owns at most one attached root layout.
- Layouts arrange views but do not replace view-tree ownership.
- An action is application-owned and may be referenced by many controls.
- A live object belongs to exactly one application.

Attaching an already attached view or layout is an error. Reparenting is not supported initially.

## Removal and destruction

`TuiContainer.Remove` destroys the removed subtree; it does not return a detachable live object. `TuiView.Destroy` is allowed for an unattached object or a subtree root and performs the same deterministic teardown.

Destruction proceeds depth-first:

1. remove the subtree from routing and layout;
2. release pointer capture and repair focus;
3. run detach and close notifications where applicable;
4. release registry entries;
5. increment generations before slots can be reused.

During `OnClosed`, the closing handle remains valid for read-only inspection. It becomes stale immediately after the callback returns.

Closing an application destroys all remaining objects. Repeating `TuiApplication.Close` for the same closed application is a no-op; every other operation on its handles fails.
