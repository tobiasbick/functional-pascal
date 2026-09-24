import type { TerminalInputEvent } from "./input";

/** Adapter protocol bound, also enforced by fpas-debug's terminalInput handler. */
export const TERMINAL_EVENT_BATCH_SIZE = 256;
const MAX_QUEUED_EVENTS = 16_384;
const MAX_QUEUED_BYTES = 4 * 1024 * 1024;

/** Owns bounded, ordered input delivery including input queued before initialization. */
export class TerminalInputQueue {
  private queue: Array<{ event: TerminalInputEvent; bytes: number }> = [];
  private head = 0;
  private bytes = 0;
  private ready = false;
  private sending = false;
  private closed = false;

  public constructor(
    private readonly send: (events: readonly TerminalInputEvent[]) => PromiseLike<unknown>,
    private readonly fail: (error: unknown) => void
  ) {}

  /** Enqueues input atomically; overflow explicitly ends the transport. */
  public push(events: readonly TerminalInputEvent[]): void {
    if (this.closed || events.length === 0) return;
    if (this.queue.length - this.head + events.length > MAX_QUEUED_EVENTS) {
      this.failed(new Error("Terminal input queue exceeds 16384 events."));
      return;
    }
    const entries = events.map(event => ({ event, bytes: Buffer.byteLength(JSON.stringify(event), "utf8") }));
    const added = entries.reduce((sum, entry) => sum + entry.bytes, 0);
    if (this.bytes + added > MAX_QUEUED_BYTES) {
      this.failed(new Error("Terminal input queue exceeds 4 MiB."));
      return;
    }
    this.queue.push(...entries);
    this.bytes += added;
    void this.drain();
  }

  /** Allows queued input to reach the initialized adapter. */
  public start(): void {
    this.ready = true;
    void this.drain();
  }

  /** Discards queued input when the owning session ends. */
  public dispose(): void {
    this.closed = true;
    this.queue = [];
    this.head = 0;
    this.bytes = 0;
  }

  private async drain(): Promise<void> {
    if (this.closed || !this.ready || this.sending) return;
    this.sending = true;
    try {
      while (!this.closed && this.head < this.queue.length) {
        const batch = this.queue.slice(this.head, this.head + TERMINAL_EVENT_BATCH_SIZE);
        await this.send(batch.map(entry => entry.event));
        if (this.closed) return;
        this.bytes -= batch.reduce((sum, entry) => sum + entry.bytes, 0);
        this.head += batch.length;
        if (this.head >= this.queue.length / 2) {
          this.queue = this.queue.slice(this.head);
          this.head = 0;
        }
      }
    } catch (error) {
      this.failed(error);
    } finally {
      this.sending = false;
    }
  }

  private failed(error: unknown): void {
    if (this.closed) return;
    this.dispose();
    this.fail(error);
  }
}
