import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-pricing',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="container py-5">
      <h2 class="mb-4">Pricing</h2>
      <div class="row">
        <div class="col-md-4">
          <div class="card p-3">
            <h5>Free</h5>
            <p>Get started with limited resources at no cost.</p>
          </div>
        </div>
        <div class="col-md-4">
          <div class="card p-3">
            <h5>Pro</h5>
            <p>Unlock advanced features and more resources.</p>
          </div>
        </div>
        <div class="col-md-4">
          <div class="card p-3">
            <h5>Enterprise</h5>
            <p>Tailored solutions for your business needs.</p>
          </div>
        </div>
      </div>
    </div>
  `,
})
export class PricingComponent {}