import { Component, signal } from '@angular/core';
import { RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet],
  template: `
    <div class="hero">
      <h1>Welcome to DropCode</h1>
    </div>
    <div class="features">
      <div class="card">Feature 1</div>
      <div class="card">Feature 2</div>
      <div class="card">Feature 3</div>
    </div>
  `,
  styles: [`
    .hero { text-align: center; }
    .features { display: flex; }
    .card { margin: 1rem; padding: 1rem; border: 1px solid #ccc; }
  `]
})
export class App {
  protected readonly title = signal('DropCode');
}