/** Detached reference calls, named constructors, and early argument-mode rejection. */

import assert from "node:assert/strict";
import * as vscode from "vscode";

import { closeAndRemoveSource, type DapMessage, startSession, waitFor, writeSource } from "./support";

export async function verifyVarParameterCalls(
  workspaceRoot: string,
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerVarParameters;",
    "uses Std.Console;",
    "type Box = record",
    "  Value: integer := 2;",
    "  static function Create(Value: integer): Box;",
    "  begin",
    "    return Box(Value := Value);",
    "  end function;",
    "  function FailFirst(Self: Box; var Item: integer): integer;",
    "  begin",
    "    panic('callee must not execute');",
    "    return Self.Value + Item;",
    "  end function;",
    "end record;",
    "function FailRoutine(var Item: integer): integer;",
    "begin",
    "  panic('callee must not execute');",
    "  return Item;",
    "end function;",
    "function Ignore(var Item: integer): integer;",
    "begin",
    "  return 42;",
    "end function;",
    "procedure FailProcedure(var Item: integer);",
    "begin",
    "  panic('callee must not execute');",
    "end procedure;",
    "function Add(var Item: integer; Amount: integer): integer;",
    "begin",
    "  Item := Item + Amount;",
    "  return Item;",
    "end function;",
    "function Increment(var Item: integer): integer;",
    "begin",
    "  Item := Item + 1;",
    "  return Item;",
    "end function;",
    "begin",
    "  var Item: integer := 1;",
    "  const Holder: Box := Box(Value := 2);",
    "  const Handler: function(var Item: integer): integer := FailRoutine;",
    "  const Bound: function(var Item: integer): integer := Holder.FailFirst;",
    "  const SafeHandler: function(var Item: integer): integer := Increment;",
    "  const StopMarker: integer := 0;",
    "  WriteLn(Item);",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "var-parameters", lines);
  const breakpoint = new vscode.SourceBreakpoint(new vscode.Location(
    vscode.Uri.file(sourcePath),
    new vscode.Position(lines.indexOf("  const StopMarker: integer := 0;"), 2)
  ));
  const marker = sent.length;
  vscode.debug.addBreakpoints([breakpoint]);
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas", request: "launch", name: "FPAS debugger var parameters",
      program: sourcePath, cwd: workspaceRoot, stopOnEntry: false
    });
    await waitFor(() => sent.slice(marker).some(message => message.event === "stopped"),
      "var-parameter evaluation stop");
    const stack = await session.customRequest("stackTrace", { threadId: 1 }) as {
      stackFrames: Array<{ id: number }>;
    };
    const frameId = stack.stackFrames[0]?.id;
    assert.ok(frameId, "var-parameter stop exposes a frame");
    const active = session;
    for (const expression of [
      "Ignore(1)", "FailRoutine(1)", "FailProcedure(1)",
      "Handler(1)", "Box.Create(2).FailFirst(1)", "Bound(1)"
    ]) {
      await assert.rejects(async () => active.customRequest("evaluate", {
        expression, frameId, context: "repl"
      }), /declares a `var` parameter/, `${expression} is rejected before its body executes`);
      const item = await active.customRequest("evaluate", {
        expression: "Item", frameId, context: "watch"
      }) as { result: string };
      assert.equal(item.result, "1", "the stopped frame and caller value remain available");
    }
    for (const [expression, expected] of [
      ["Ignore(var Item)", "42"],
      ["Add(var Item, 3)", "4"],
      ["Add(Amount := 3, Item := var Item)", "4"],
      ["SafeHandler(var Item)", "2"],
      ["Box(Value := 9).Value", "9"],
      ["Box().Value", "2"],
      ["Box.Create(Value := 8).Value", "8"]
    ]) {
      const result = await active.customRequest("evaluate", { expression, frameId, context: "watch" }) as { result: string };
      assert.equal(result.result, expected, expression);
      const item = await active.customRequest("evaluate", { expression: "Item", frameId, context: "watch" }) as { result: string };
      assert.equal(item.result, "1", "reference calls preserve the live caller variable");
    }
    await assert.rejects(async () => active.customRequest("evaluate", {
      expression: "SafeHandler(Item := var Item)", frameId, context: "watch"
    }), /function values accept only positional arguments/);
    await session.customRequest("continue", { threadId: 1 });
    await waitFor(() => sent.slice(marker).some(message => message.event === "terminated"),
      "var-parameter session termination");
    assert.equal(sent.slice(marker).filter(message => message.event === "output")
      .map(message => String(message.body?.output ?? "")).join("").trim(), "1");
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
  await verifyVarParameterWrites(workspaceRoot, sent);
}


async function verifyVarParameterWrites(workspaceRoot: string, sent: DapMessage[]): Promise<void> {
  const lines = [
    "program DebuggerVarWrites;",
    "uses Std.Console;",
    "var Original: integer := 1;",
    "procedure Change(var Item: integer);",
    "begin",
    "  const WriteMarker: integer := 0;",
    "  Item := Item + 1;",
    "end procedure;",
    "begin",
    "  Change(var Original);",
    "  WriteLn(Original);",
    "end.", ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "var-writes", lines);
  const breakpoint = new vscode.SourceBreakpoint(new vscode.Location(
    vscode.Uri.file(sourcePath), new vscode.Position(lines.indexOf("  const WriteMarker: integer := 0;"), 2)
  ));
  const marker = sent.length;
  vscode.debug.addBreakpoints([breakpoint]);
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas", request: "launch", name: "FPAS debugger var writes",
      program: sourcePath, cwd: workspaceRoot, stopOnEntry: false
    });
    await waitFor(() => sent.slice(marker).some(message => message.event === "stopped"), "var write stop");
    const active = session;
    const frame = async (): Promise<number> => {
      const stack = await active.customRequest("stackTrace", { threadId: 1 }) as { stackFrames: Array<{ id: number }> };
      assert.ok(stack.stackFrames[0]?.id, "var parameter frame");
      return stack.stackFrames[0].id;
    };
    const frameId = await frame();
    await assert.rejects(async () => active.customRequest("setExpression", {
      frameId, expression: "Item", value: "'wrong'"
    }), /type|match/);
    const scopes = await active.customRequest("scopes", { frameId }) as {
      scopes: Array<{ name: string; variablesReference: number }>;
    };
    const parameters = scopes.scopes.find(scope => scope.name === "Parameters");
    assert.ok(parameters, "var parameter scope survives a failed assignment");
    const replacement = await active.customRequest("setVariable", {
      variablesReference: parameters.variablesReference, name: "Item", value: "9"
    }) as { value: string };
    assert.equal(replacement.value, "9");
    const original = await active.customRequest("evaluate", {
      expression: "Original", frameId: await frame(), context: "watch"
    }) as { result: string };
    assert.equal(original.result, "9", "setVariable writes through the parameter reference");
    const result = await active.customRequest("setExpression", {
      frameId: await frame(), expression: "Item", value: "10"
    }) as { value: string };
    assert.equal(result.value, "10", "setExpression uses the same referent storage");
    await active.customRequest("continue", { threadId: 1 });
    await waitFor(() => sent.slice(marker).some(message => message.event === "terminated"), "var write termination");
    assert.equal(sent.slice(marker).filter(message => message.event === "output")
      .map(message => String(message.body?.output ?? "")).join("").trim(), "11");
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
