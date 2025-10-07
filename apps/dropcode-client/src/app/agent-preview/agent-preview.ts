import {Component, OnDestroy, ChangeDetectorRef, ViewChild, ElementRef, OnInit} from '@angular/core';
import { RouterLink } from '@angular/router';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';
import {NgForOf, NgIf} from "@angular/common";
import {ChatMessage} from "./chat-message/chat-message";
import {Thinking} from "./thinking/thinking";
import {Terminal} from "./terminal/terminal";
import {ChatAnimationService} from "./chat-animation-service";

type QueueItem = { type: string, template?: string; delay?: number; meta?: any };

@Component({
  selector: 'app-agent-preview',
  imports: [NgForOf, NgIf, ChatMessage, Thinking, Terminal],
  templateUrl: './agent-preview.html',
  styleUrl: './agent-preview.scss'
})
export class AgentPreview implements OnInit {
  @ViewChild('scrollMe')
  private agentPreviewChat!: ElementRef

  /** items you want rendered in the template (you'll *ngFor this) */
  model: QueueItem[] = [];

  /** you fill this from outside or in ngOnInit; left empty per your request */
  queue: QueueItem[] = [
    {
      type: 'agent',
      meta: 'HermesAI',
      template: `Hi! What are we building today?`
    },
    {
      type: 'user',
      meta: 'You • now',
      template: `Make me a full-stack <strong>Angular 20</strong> + <strong>FlaskAPI</strong> + <strong>PostgreSQL</strong>
          application and give me a <strong>docker compose</strong> entrypoint to spin everything up.`
    },
    {
      type: 'thinking',
      template: 'Planning…',
    },
    {
      type: 'thinking',
      template: 'Reviewing plan with Architect…',
    },
    {
      type: 'agent',
      meta: 'HermesAI • plan ready',
      template: `Here's the plan!
##### Summary
- **Frontend:** Angular 20 (standalone, SCSS, routing) served via Vite dev in compose.
- **API:** Flask + Gunicorn, connects via <code>DATABASE_URL</code>.
- **DB:** Postgres 16 with named volume <code>acme_pgdata</code>.
- **Compose:** <code>web</code> ↔ <code>api</code> ↔ <code>db</code> with healthchecks; hot reload for dev.
- **CI:** basic workflow: install → lint/test → build docker images.
If this sounds good, I'll implement this architecture in the live sandbox for you to preview now.`
    },
    {
      type: 'user',
      meta: 'You • now',
      template: `Looks great, go for it!`
    },
    {
      type: 'thinking',
      template: 'Converting plan to file-level task list…',
    },
    {
      type: 'thinking',
      template: 'Reviewing file-level changes with Specialists…',
    },
    {
      type: 'agent',
      meta: 'HermesAI • Specialist Recommendations',
      template: `I've reviewed the plan thoroughly. I see that we need to pick our peer dependency versions carefully with Angular projects to ensure compatibility between TypeScript, ESLint, Jest/Karma, and Angular. I'll confirm the exact versions now.`
    },
      /* todo: Show Agentic Workforce Process here
        - expandable thinking steps
        - show the cross referencing / web-searching / error-reduction / lang-specific agentic specialization
      */
      /* todo: Show the File-Level Task result & User review process
      */
    {
      type: 'terminal',
    },
    {
      type: 'agent',
      meta: 'HermesAI • plan ready',
      template: `Done! The preview is live in the [sandbox](https://hermesai.dev/early-access). **✓**`
    },

  ];

  constructor(
      private sanitize: DomSanitizer,
      private cdr: ChangeDetectorRef,
      private chatAnimation: ChatAnimationService,
  ) {
  }

  safe(html: string): SafeHtml {
    return this.sanitize.bypassSecurityTrustHtml(html);
  }

  ngOnInit() {
    // let each element in the Chat Animation call ChatAnimationService.notifyDone() to trigger the next chat
    // this will allow typing animation etc.
    let chatFrame = 0;
    this.model.push(this.queue[chatFrame]);

    let scrollLoop = setInterval(() => {
      this.agentPreviewChat.nativeElement.scrollTo({ top: this.agentPreviewChat.nativeElement.scrollHeight, behavior: 'smooth' });
    }, 1500);

    let restart = false;

    this.chatAnimation.done$.subscribe(() => {
      this.agentPreviewChat.nativeElement.scrollTo({ top: this.agentPreviewChat.nativeElement.scrollHeight, behavior: 'smooth' });
      chatFrame += 1;

      const currentFrame: QueueItem = this.queue[chatFrame];
      if (!currentFrame) {
        clearInterval(scrollLoop);
        return;
      }
      this.model.push(currentFrame);

      if (currentFrame.type === 'terminal') {
        restart = true;
        clearInterval(scrollLoop);
        scrollLoop = setTimeout(() => {
          this.agentPreviewChat.nativeElement.scrollTo({ top: this.agentPreviewChat.nativeElement.scrollTop + 1, behavior: 'smooth' });
        }, 100);
      }
      else {
        setTimeout(() => {
          this.agentPreviewChat.nativeElement.scrollTo({ top: this.agentPreviewChat.nativeElement.scrollHeight, behavior: 'smooth' });
        });
        if (restart) {
          clearInterval(scrollLoop);
          scrollLoop = setInterval(() => {
            this.agentPreviewChat.nativeElement.scrollTo({ top: this.agentPreviewChat.nativeElement.scrollHeight, behavior: 'smooth' });
          }, 1000);
        }
      }

    });
  }
}
