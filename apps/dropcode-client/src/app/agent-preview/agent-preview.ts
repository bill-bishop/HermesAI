import {Component, OnDestroy, ChangeDetectorRef} from '@angular/core';
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
export class AgentPreview {
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
          <div class="plan">
            <div class="plan-title">Summary</div>
            <ul>
              <li><strong>Frontend:</strong> Angular 20 (standalone, SCSS, routing) served via Vite dev in compose.</li>
              <li><strong>API:</strong> Flask + Gunicorn, connects via <code>DATABASE_URL</code>.</li>
              <li><strong>DB:</strong> Postgres 16 with named volume <code>acme_pgdata</code>.</li>
              <li><strong>Compose:</strong> <code>web</code> ↔ <code>api</code> ↔ <code>db</code> with healthchecks; hot reload for dev.</li>
              <li><strong>CI:</strong> basic workflow: install → lint/test → build docker images.</li>
            </ul>
          </div>
          <div class="cta">
            If this sounds good, I'll implement this architecture in the live sandbox for you to preview now.
          </div>`
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
      template: `<span>Done! The preview is live in the sandbox.</span>
          <span class="check" aria-label="success" style="
    display: inline-grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid var(--lime);
    color: var(--lime);
    font-weight: 800;
    line-height: 1;">✓</span>
          <a class="btn btn-primary btn-sm ms-auto" style="
    margin-left: 2em !important;
    font-size: 0.78rem;
    padding: 0.3rem 0.55rem;
    border-radius: 0.5rem;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: linear-gradient(180deg, rgba(84, 122, 255, 0.22), rgba(84, 122, 255, 0.08));
    color: #e8e9f0;
    cursor: pointer;
">Preview</a>`
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

    this.chatAnimation.done$.subscribe(() => {
      chatFrame += 1;
      this.model.push(this.queue[chatFrame])
    });
  }
}
