/** VT input decoding for the debugger-owned VS Code pseudoterminal. */

export interface TerminalKeyEvent {
  readonly kind: "key";
  readonly key: string;
  readonly text?: string;
  readonly shift?: boolean;
  readonly ctrl?: boolean;
  readonly alt?: boolean;
  readonly meta?: boolean;
}

export interface TerminalMouseEvent {
  readonly kind: "mouse";
  readonly action: string;
  readonly button: string;
  readonly x: number;
  readonly y: number;
  readonly shift?: boolean;
  readonly ctrl?: boolean;
  readonly alt?: boolean;
  readonly meta?: boolean;
}

export type TerminalInputEvent =
  | TerminalKeyEvent
  | TerminalMouseEvent
  | { readonly kind: "resize"; readonly width: number; readonly height: number }
  | { readonly kind: "paste"; readonly text: string }
  | { readonly kind: "focusGained" | "focusLost" };

const PASTE_START = "\u001b[200~";
const PASTE_END = "\u001b[201~";

/** Maximum retained paste or input fragment size in UTF-8 bytes. */
export const MAX_TERMINAL_INPUT_BYTES = 1024 * 1024;
const MAX_CONTROL_LENGTH = 128;

/** Incrementally decodes a byte-stream's text without treating chunks as keys. */
export class TerminalInputDecoder {
  private pending = "";
  private paste: string[] | undefined;
  private pasteBytes = 0;

  /** Emits a lone Escape after the transport's ambiguity deadline. */
  flushEscape(): TerminalInputEvent[] {
    if (this.pending !== "\u001b" || this.paste !== undefined) return [];
    this.pending = "";
    return [keyEvent("Escape")];
  }

  /** Whether the transport must schedule the lone-Escape deadline. */
  get waitingForEscape(): boolean {
    return this.pending === "\u001b" && this.paste === undefined;
  }

  /** Discards retained input when the transport closes or input exceeds its bound. */
  reset(): void {
    this.pending = "";
    this.paste = undefined;
    this.pasteBytes = 0;
  }

  /** Decodes complete sequences and retains only their incomplete suffix. */
  feed(data: string): TerminalInputEvent[] {
    if (Buffer.byteLength(data, "utf8") > MAX_TERMINAL_INPUT_BYTES) {
      this.reset();
      throw new Error("Terminal input fragment exceeds the 1 MiB limit.");
    }
    const input = this.pending + data;
    this.pending = "";
    const events: TerminalInputEvent[] = [];
    let cursor = 0;
    while (cursor < input.length) {
      if (this.paste !== undefined) {
        const end = input.indexOf(PASTE_END, cursor);
        if (end < 0) {
          // Only a possible delimiter prefix must survive into the next feed.
          let keep = Math.min(PASTE_END.length - 1, input.length - cursor);
          while (keep > 0 && !PASTE_END.startsWith(input.slice(input.length - keep))) keep -= 1;
          this.appendPaste(input.slice(cursor, input.length - keep));
          cursor = input.length - keep;
          break;
        }
        this.appendPaste(input.slice(cursor, end));
        events.push({ kind: "paste", text: this.paste.join("") });
        this.paste = undefined;
        this.pasteBytes = 0;
        cursor = end + PASTE_END.length;
        continue;
      }
      if (input[cursor] !== "\u001b") {
        const code = input.codePointAt(cursor)!;
        if (code >= 0xd800 && code <= 0xdbff && cursor + 1 === input.length) break;
        const character = String.fromCodePoint(code);
        events.push(characterEvent(character));
        cursor += character.length;
        continue;
      }
      const rest = input.slice(cursor);
      if (rest.length === 1) break;
      if (rest[1] === "[" || rest[1] === "O") {
        // CSI parameters/intermediates end at one final byte; unsupported finals are consumed.
        const sequence = /^\u001b([\[O])([ -?]*)([@-~])/u.exec(rest);
        if (sequence === null) {
          if (rest.length > MAX_CONTROL_LENGTH) {
            this.reset();
            throw new Error("Terminal control sequence exceeds 128 characters.");
          }
          const invalid = /[^ -?]/u.exec(rest.slice(2));
          if (invalid !== null) {
            cursor += 2 + invalid.index;
            continue;
          }
          break;
        }
        const [wire, prefix, parameters, final] = sequence;
        cursor += wire.length;
        if (wire.length > MAX_CONTROL_LENGTH) continue;
        if (wire === PASTE_START) { this.paste = []; continue; }
        if (prefix === "O") {
          if (parameters === "" && "PQRS".includes(final)) events.push(keyEvent(`F${"PQRS".indexOf(final) + 1}`));
          else if (parameters === "" && "ABCDHF".includes(final)) events.push(csiKeyEvent(undefined, undefined, final)!);
          continue;
        }
        if (parameters === "" && (final === "I" || final === "O")) {
          events.push({ kind: final === "I" ? "focusGained" : "focusLost" });
          continue;
        }
        const mouse = /^<(\d+);(\d+);(\d+)$/u.exec(parameters);
        if (mouse !== null && (final === "M" || final === "m")) {
          const values = mouse.slice(1).map(Number);
          if (values.every(Number.isSafeInteger)) events.push(mouseEvent(values[0], values[1], values[2], final));
          continue;
        }
        const key = /^(?:(\d+)(?:;(\d+))?)?$/u.exec(parameters);
        if (key !== null) {
          const event = csiKeyEvent(key[1], key[2], final);
          if (event !== undefined) events.push(event);
        }
        continue;
      }
      const code = input.codePointAt(cursor + 1)!;
      if (code >= 0xd800 && code <= 0xdbff && cursor + 2 === input.length) break;
      const character = String.fromCodePoint(code);
      events.push(keyEvent("Character", character, { alt: true }));
      cursor += 1 + character.length;
    }
    this.pending = input.slice(cursor);
    return events;
  }

  private appendPaste(text: string): void {
    this.pasteBytes += Buffer.byteLength(text, "utf8");
    if (this.pasteBytes > MAX_TERMINAL_INPUT_BYTES) {
      this.reset();
      throw new Error("Terminal paste exceeds the 1 MiB limit.");
    }
    if (text.length > 0) this.paste!.push(text);
  }
}

function characterEvent(character: string): TerminalKeyEvent {
  if (character === "\r" || character === "\n") return keyEvent("Enter");
  if (character === "\t") return keyEvent("Tab");
  if (character === "\u007f" || character === "\b") return keyEvent("Backspace");
  if (character === " ") return keyEvent("Space");
  const code = character.codePointAt(0) ?? 0;
  if (code >= 1 && code <= 26) {
    return keyEvent("Character", String.fromCharCode(96 + code), { ctrl: true });
  }
  return keyEvent("Character", character);
}

function csiKeyEvent(
  parameter: string | undefined,
  modifier: string | undefined,
  suffix: string
): TerminalKeyEvent | undefined {
  const modifiers = keyModifiers(Number(modifier ?? 1));
  if (suffix === "Z") {
    return keyEvent("Tab", undefined, { ...modifiers, shift: true });
  }
  const directional: Record<string, string> = {
    A: "Up",
    B: "Down",
    C: "Right",
    D: "Left",
    H: "Home",
    F: "End"
  };
  if (suffix !== "~") {
    const key = directional[suffix];
    return key === undefined ? undefined : keyEvent(key, undefined, modifiers);
  }
  const numbered: Record<string, string> = {
    "1": "Home",
    "2": "Insert",
    "3": "Delete",
    "4": "End",
    "5": "PageUp",
    "6": "PageDown",
    "15": "F5",
    "17": "F6",
    "18": "F7",
    "19": "F8",
    "20": "F9",
    "21": "F10",
    "23": "F11",
    "24": "F12"
  };
  const key = numbered[parameter ?? ""];
  return key === undefined ? undefined : keyEvent(key, undefined, modifiers);
}

function keyModifiers(value: number): Omit<TerminalKeyEvent, "kind" | "key" | "text"> {
  const bits = Math.max(1, value) - 1;
  return {
    shift: (bits & 1) !== 0,
    alt: (bits & 2) !== 0,
    ctrl: (bits & 4) !== 0,
    meta: (bits & 8) !== 0
  };
}

function keyEvent(
  key: string,
  text?: string,
  modifiers: Omit<TerminalKeyEvent, "kind" | "key" | "text"> = {}
): TerminalKeyEvent {
  return { kind: "key", key, ...(text === undefined ? {} : { text }), ...modifiers };
}

function mouseEvent(code: number, x: number, y: number, suffix: string): TerminalMouseEvent {
  const buttonCode = code & 3;
  const button = ["Left", "Middle", "Right", "None"][buttonCode];
  let action: string;
  if ((code & 64) !== 0) {
    action = ["ScrollUp", "ScrollDown", "ScrollLeft", "ScrollRight"][buttonCode];
  } else if (suffix === "m") {
    action = "Up";
  } else if ((code & 32) !== 0) {
    action = buttonCode === 3 ? "Move" : "Drag";
  } else if (buttonCode === 3) {
    action = "Up";
  } else {
    action = "Down";
  }
  return {
    kind: "mouse",
    action,
    button: action.startsWith("Scroll") || action === "Move" ? "None" : button,
    x,
    y,
    shift: (code & 4) !== 0,
    alt: (code & 8) !== 0,
    ctrl: (code & 16) !== 0,
    meta: false
  };
}
