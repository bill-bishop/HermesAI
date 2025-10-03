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

    <div class="container my-5">
      <div class="row justify-content-center g-4">
        <!-- Terminal Card -->
        <div class="col-md-6">
          <div class="card shadow-sm border-0">
            <img
              src="https://dropcode.org/assets/terminal.png"
              class="card-img-top"
              alt="DropCode Terminal"
            />
            <div class="card-body text-center">
              <h5 class="card-title">DropCode Terminal</h5>
              <p class="card-text">
                Try out the DropCode interactive terminal directly in your browser.
              </p>
              <a
                href="https://dropcode.org/terminal.html"
                class="btn btn-primary"
                target="_blank"
                rel="noopener noreferrer"
              >
                Open Terminal
              </a>
            </div>
          </div>
        </div>

        <!-- HermesAI Assistant Card -->
        <div class="col-md-6">
          <div class="card shadow-sm border-0">
            <img
              src="https://dropcode.org/assets/agent.png"
              class="card-img-top"
              alt="HermesAI Executive Assistant"
            />
            <div class="card-body text-center">
              <p class="card-text">
                Meet HermesAI — your executive programming assistant for DropCode.
              </p>
              <a
                href="https://chatgpt.com/g/g-67659602f22c81919da3c6b22b96658c-hermesai-executive-gpt"
                class="btn btn-primary"
                target="_blank"
                rel="noopener noreferrer"
              >
                Launch HermesAI
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  `,
})
export class HomeComponent {}
