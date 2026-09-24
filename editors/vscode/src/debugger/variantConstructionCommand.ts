/** Interactive complete-variant construction over the FPAS DAP adapter. */

import * as vscode from "vscode";
import { activeSelection, type DebugSelection } from "./commands/selection";
import { promptExpression as prompt } from "./commands/expressionPrompt";

/** Stable command identifier contributed by the Functional Pascal extension. */
export const CONSTRUCT_VARIANT_COMMAND = "functionalPascal.debug.constructVariant";

/** Optional arguments used by command links and Extension Host tests. */
export interface VariantConstructionInput {
  readonly frameId?: number;
  readonly target?: string;
  readonly variant?: string;
  readonly fields?: Record<string, string>;
}

interface VariantField {
  readonly name: string;
  readonly typeName: string;
}

interface VariantInfo {
  readonly name: string;
  readonly fields: readonly VariantField[];
}

interface VariantDescription {
  readonly target?: string;
  readonly typeName?: string;
  readonly variants?: readonly VariantInfo[];
}

/** Register the editor command that constructs one complete wrapper variant. */
export function registerVariantConstructionCommand(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      CONSTRUCT_VARIANT_COMMAND,
      async (input?: VariantConstructionInput) => {
        const selection = await activeSelection(input?.frameId, "constructing a variant");
        if (selection === undefined) return;
        try {
          const target = input?.target ?? await prompt("Mutable enum, Result, or Option target", "Selected");
          if (target === undefined) return;
          const description = await selection.session.customRequest("fpas/variantDescribe", {
            frameId: selection.frameId,
            target
          }) as VariantDescription;
          const variants = description.variants ?? [];
          const selected = input?.variant !== undefined
            ? variants.find((variant) => variant.name.toLowerCase() === input.variant?.toLowerCase())
              ?? { name: input.variant, fields: [] }
            : await pickVariant(variants);
          if (selected === undefined) return;
          const fields = unmatchedFields(input?.fields, selected.fields);
          for (const field of selected.fields) {
            const expression = input?.fields?.[field.name]
              ?? Object.entries(input?.fields ?? {}).find(([name]) => name.toLowerCase() === field.name.toLowerCase())?.[1]
              ?? await prompt(`${selected.name} field ${field.name}: ${field.typeName}`, "0");
            if (expression === undefined) return;
            fields[field.name] = expression;
          }
          const result = await selection.session.customRequest("fpas/variantConstruct", {
            frameId: selection.frameId,
            target,
            variant: selected.name,
            fields
          }) as { value?: string; variant?: string };
          void vscode.window.showInformationMessage(
            `Functional Pascal constructed ${result.variant ?? selected.name}: ${result.value ?? "committed"}`
          );
        } catch (error) {
          void vscode.window.showErrorMessage(
            `Functional Pascal variant construction failed: ${errorMessage(error)}`
          );
          if (input !== undefined) throw error;
        }
      }
    )
  );
}

async function pickVariant(variants: readonly VariantInfo[]): Promise<VariantInfo | undefined> {
  const picked = await vscode.window.showQuickPick(
    variants.map((variant) => ({
      label: variant.name,
      description: variant.fields.map((field) => field.name).join(", "),
      variant
    })),
    {
      placeHolder: "Select a variant",
      ignoreFocusOut: true
    }
  );
  return picked?.variant;
}


function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function unmatchedFields(
  supplied: Record<string, string> | undefined,
  expected: readonly VariantField[]
): Record<string, string> {
  return Object.fromEntries(
    Object.entries(supplied ?? {}).filter(([name]) =>
      !expected.some((field) => field.name.toLowerCase() === name.toLowerCase())
    )
  );
}
