import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-features',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="container py-5">
      <h2 class="mb-4">Features</h2>
      <div class="row">
        <div class="col-md-4">
          <div class="card p-3">
            <h5>Fast Execution</h5>
            <p>Run code instantly in isolated sandboxes.</p>
          </div>
        </div>
        <div class="col-md-4">
          <div class="card p-3">
            <h5>Collaboration</h5>
            <p>Work with your team in real time.</p>
          </div>
        </div>
        <div class="col-md-4">
          <div class="card p-3">
            <h5>Deploy Anywhere</h5>
            <p>Export and deploy your projects with one click.</p>
          </div>
        </div>
      </div>
    </div>
  `,
})
export class FeaturesComponent {}