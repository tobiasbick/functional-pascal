import { TerminalInputDecoder, type TerminalInputEvent } from "./input";

/** Resolves lone Escape after 50 ms while allowing split VT sequences to complete. */
export class TimedTerminalInput {
  private readonly decoder = new TerminalInputDecoder();
  private timer: ReturnType<typeof setTimeout> | undefined;
  private disposed = false;

  public constructor(
    private readonly send: (events: TerminalInputEvent[]) => void,
    private readonly fail: (error: unknown) => void
  ) {}

  /** Feeds text and resets the ambiguity timer only when Escape is incomplete. */
  public feed(text: string): void {
    if (this.disposed) return;
    if (this.timer !== undefined) clearTimeout(this.timer);
    this.timer = undefined;
    try {
      this.send(this.decoder.feed(text));
      if (this.decoder.waitingForEscape) {
        this.timer = setTimeout(() => {
          this.timer = undefined;
          this.send(this.decoder.flushEscape());
        }, 50);
      }
    } catch (error) {
      this.dispose();
      this.fail(error);
    }
  }

  /** Cancels delayed input and releases any retained paste or control prefix. */
  public dispose(): void {
    this.disposed = true;
    if (this.timer !== undefined) clearTimeout(this.timer);
    this.timer = undefined;
    this.decoder.reset();
  }
}
