import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { AuthService } from '../auth.service';
import { ActivatedRoute } from '@angular/router';

@Component({
  selector: 'app-third-party-auth',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './third-party-auth.component.html',
  styleUrls: ['./third-party-auth.component.scss']
})
export class ThirdPartyAuthComponent {
  @Input() providers: string[] = ['github'];
  public target?: string;

  constructor(private authService: AuthService, private route: ActivatedRoute) {
    // Subscribe to query params so we always capture target
    this.route.queryParamMap.subscribe(params => {
      this.target = params.get('target') ?? undefined;
    });
  }

  loginWith(provider: string) {
    switch (provider) {
      case 'github':
        this.authService.loginWithGithub(this.target);
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