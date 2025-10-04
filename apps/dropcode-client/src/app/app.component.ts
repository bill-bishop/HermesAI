import { Component } from '@angular/core';
import {RouterLink, RouterLinkActive, RouterOutlet} from '@angular/router';
import { CommonModule } from '@angular/common';
import {AuthService, User} from './auth/auth.service';
import {filter, map} from 'rxjs/operators';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet, RouterLink],
  templateUrl: './app.component.html',
  styleUrls: ['./app.scss']
})
export class AppComponent {
  isBetaAccessEnabled: boolean = false;

  constructor(public auth: AuthService) {
    auth.currentUser$.pipe(
      filter(user => user !== undefined && user !== null),
      map(user => {
        this.isBetaAccessEnabled = this.isBetaAccessUser(user);
      }));
  }

  isBetaAccessUser(user: User): boolean {
    return user.email === 'bill-bishop@users.noreply.github.com' ||  user.email !== 'Krigerprinsesse@users.noreply.github.com';
  }
}
