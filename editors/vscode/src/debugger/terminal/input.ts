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

/** Incrementally decode terminal input chunks into debugger terminal events. */
export class TerminalInputDecoder {
  private pending = "";

  /** Decode one input chunk while retaining incomplete control sequences. */
  feed(data: string): TerminalInputEvent[] {
    this.pending += data;
    const events: TerminalInputEvent[] = [];
    while (this.pending.length > 0) {
      if (!this.pending.startsWith("\u001b")) {
        const character = [...this.pending][0];
        this.pending = this.pending.slice(character.length);
        events.push(characterEvent(character));
        continue;
      }
      if (this.pending.startsWith(PASTE_START)) {
        const end = this.pending.indexOf(PASTE_END, PASTE_START.length);
        if (end < 0) {
          break;
        }
        events.push({
          kind: "paste",
          text: this.pending.slice(PASTE_START.length, end)
        });
        this.pending = this.pending.slice(end + PASTE_END.length);
        continue;
      }
      const focus = /^\u001b\[([IO])/u.exec(this.pending);
      if (focus !== null) {
        events.push({ kind: focus[1] === "I" ? "focusGained" : "focusLost" });
        this.pending = this.pending.slice(focus[0].length);
        continue;
      }
      const mouse = /^\u001b\[<(\d+);(\d+);(\d+)([Mm])/u.exec(this.pending);
      if (mouse !== null) {
        events.push(mouseEvent(Number(mouse[1]), Number(mouse[2]), Number(mouse[3]), mouse[4]));
        this.pending = this.pending.slice(mouse[0].length);
        continue;
      }
      const csiKey = /^\u001b\[(?:(\d+)(?:;(\d+))?)?([A-DHFZ~])/u.exec(this.pending);
      if (csiKey !== null) {
        const event = csiKeyEvent(csiKey[1], csiKey[2], csiKey[3]);
        if (event !== undefined) {
          events.push(event);
          this.pending = this.pending.slice(csiKey[0].length);
          continue;
        }
      }
      const ss3 = /^\u001bO([PQRS])/u.exec(this.pending);
      if (ss3 !== null) {
        events.push(keyEvent(`F${"PQRS".indexOf(ss3[1]) + 1}`));
        this.pending = this.pending.slice(ss3[0].length);
        continue;
      }
      if (this.pending === "\u001b" || this.pending === "\u001b[") {
        if (this.pending === "\u001b[") {
          break;
        }
        events.push(keyEvent("Escape"));
        this.pending = "";
        continue;
      }
      const character = [...this.pending.slice(1)][0];
      if (character !== "[") {
        events.push(keyEvent("Character", character, { alt: true }));
        this.pending = this.pending.slice(1 + character.length);
        continue;
      }
      break;
    }
    return events;
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
  } else if (suffix === "m" || buttonCode === 3) {
    action = "Up";
  } else if ((code & 32) !== 0) {
    action = buttonCode === 3 ? "Move" : "Drag";
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
