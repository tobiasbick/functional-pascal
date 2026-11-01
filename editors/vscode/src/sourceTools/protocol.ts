/** Snapshot-backed source inspection returned by the language server. */
export interface SourceLocation {
  readonly uri: string;
  readonly range: { readonly start: Position; readonly end: Position };
  readonly byte_start: number;
  readonly byte_length: number;
}
export interface Position { readonly line: number; readonly character: number }
export interface InspectRequest {
  readonly version: 1;
  readonly operation: "inspect";
  readonly context?: string;
  readonly selector: { readonly kind: "position"; readonly uri: string; readonly position: Position };
}
export interface InspectionSection {
  readonly kind: "signature" | "implementation" | "supporting_type";
  readonly reason: string;
  readonly qualified_name: string;
  readonly origin: string;
  readonly checked: boolean;
  readonly location: SourceLocation | null;
  readonly detail: string | null;
  readonly documentation: string | null;
  readonly full_bytes: number;
  readonly omitted_bytes: number;
  readonly excerpts: readonly { readonly location: SourceLocation; readonly text: string }[];
}
export interface InspectResponse {
  readonly version: 1;
  readonly outcome: "ok" | "not_found" | "incomplete" | "error";
  readonly target: { readonly qualified_name: string } | null;
  readonly context: { readonly root: string; readonly standard_library: string | null };
  readonly snapshot: { readonly inventory: {
    readonly files: readonly { readonly path: string }[];
    readonly aliases: readonly (readonly [string, string])[];
    readonly globs: readonly { readonly root: string }[];
  } };
  readonly result: {
    readonly kind: "inspect";
    readonly sections: readonly InspectionSection[];
    readonly omissions: readonly { readonly section: string; readonly reason: string; readonly bytes: number | null }[];
  };
  readonly coverage: { readonly analysis_complete: boolean; readonly reasons: readonly string[] };
  readonly diagnostics: readonly { readonly message: string; readonly code?: string }[];
}
