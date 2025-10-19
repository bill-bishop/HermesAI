import {
  Component,
  Input,
  OnInit,
  OnDestroy,
  ChangeDetectionStrategy,
  signal, Output, EventEmitter,
} from '@angular/core';
import { MarkdownComponent } from 'ngx-markdown';
import { createUnifiedStream, toClosingTag } from 'markityper';

@Component({
  selector: 'markityper',
  standalone: true,
  imports: [MarkdownComponent],
  template: `<markdown [data]="typed() + untypedClosingTags"></markdown>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MarkityperComponent implements OnInit, OnDestroy {
  /** Full markdown string to stream */
  @Input() text = '';

  /** Letters per second — controls typing pace */
  @Input() lps = 20;

  @Output()
  onComplete: EventEmitter<boolean> = new EventEmitter();

  untypedClosingTags: string[] = [];

  /** Signal that holds the progressively typed markdown */
  readonly typed = signal('');

  private cancelled = false;

  async ngOnInit() {
    let buffer = '';

    try {
      for await (const tok of createUnifiedStream(this.text, { gfm: true })) {
        if (this.cancelled) return;

        const { type, kind, value } = tok;

        if (type === 'syntax' && kind === 'open') {
          if (/^<.+>$/.test(value)) {
            // html syntax
            this.untypedClosingTags.push(toClosingTag(value));
          }
          else {
             // markdown syntax
            this.untypedClosingTags.push(value);
          }
        }
        else if (type === 'syntax' && kind === 'close') {
          this.untypedClosingTags.pop();
        }

        buffer += value;
        this.typed.set(buffer);
      }
      this.onComplete.emit(true);
    } catch {
      this.typed.set(this.text); // fallback: dump all text
      this.onComplete.emit(false);
    }
  }

  ngOnDestroy() {
    this.cancelled = true;
  }
}
