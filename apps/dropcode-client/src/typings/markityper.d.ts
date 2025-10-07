// markityper.d.ts
declare module 'markityper' {
  export type MarkityperToken = { type: 'syntax'; kind: 'line' | 'open' | 'close' | 'default'; value: string }

  export function createUnifiedStream(
    src: string,
    options?: Record<string, unknown>
  ): AsyncIterable<MarkityperToken>;

  export function toClosingTag(input: string): string;
}
