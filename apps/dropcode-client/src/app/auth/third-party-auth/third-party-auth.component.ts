import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-third-party-auth',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './third-party-auth.component.html',
  styleUrls: ['./third-party-auth.component.scss']
})
export class ThirdPartyAuthComponent {
  @Input() providers: string[] = ['github'];

  loginWith(provider: string) {
    switch (provider) {
      case 'github':
        // Proper GitHub OAuth flow usually starts with redirecting to backend route
        // which handles OAuth handshake. UI just kicks off the redirect.
        window.location.href = '/api/auth/github';
        break;
      case 'google':
        window.location.href = '/api/auth/google';
        break;
      case 'facebook':
        window.location.href = '/api/auth/facebook';
        break;
      default:
        console.warn(`Unknown provider: ${provider}`);
    }
  }
}