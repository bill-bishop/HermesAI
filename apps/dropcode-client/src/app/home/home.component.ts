import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="hero text-center py-5 bg-light">
      <h1 class="display-4">Welcome to DropCode</h1>
      <p class="lead">Build, test, and deploy code seamlessly in your browser.</p>
    </div>
  `,
})
export class HomeComponent {}