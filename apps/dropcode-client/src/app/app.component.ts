import { Component } from '@angular/core';

@Component({
  selector: 'app-root',
  template: `
    <div class="hero">
      <h1>Hero Headline</h1>
    </div>
    <div class="features">
      <div class="feature-card">Feature 1</div>
      <div class="feature-card">Feature 2</div>
      <div class="feature-card">Feature 3</div>
    </div>
  `,
  styles: [`
    .hero { text-align: center; }
    .features { display: flex; }
    .feature-card { margin: 1rem; padding: 1rem; border: 1px solid #ccc; }
  `]
})
export class AppComponent {
  title = 'dropcode-client';
}