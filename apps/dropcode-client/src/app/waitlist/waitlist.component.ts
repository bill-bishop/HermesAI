import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { WaitlistService } from './waitlist.service';

@Component({
  selector: 'app-waitlist',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './waitlist.component.html',
  styleUrls: ['./waitlist.component.scss']
})
export class WaitlistComponent {
  email = '';
  message = '';
  loading = false;
  showToast = false;
  toastType: 'success' | 'error' = 'success';

  constructor(private waitlistService: WaitlistService) {}

  joinWaitlist() {
    if (!this.email) {
      this.showToastMessage('Please enter a valid email address.', 'error');
      return;
    }

    this.loading = true;
    this.waitlistService.joinWaitlist(this.email).subscribe({
      next: (res) => {
        this.showToastMessage(res.message || 'Welcome to the waitlist!', 'success');
        this.email = '';
        this.loading = false;
      },
      error: (err) => {
        if (err.status === 409) {
          this.showToastMessage('You’re already on the list!', 'error');
        } else {
          this.showToastMessage('Something went wrong. Please try again later.', 'error');
        }
        this.loading = false;
      }
    });
  }

  showToastMessage(message: string, type: 'success' | 'error') {
    this.message = message;
    this.toastType = type;
    this.showToast = true;
    setTimeout(() => (this.showToast = false), 4000);
  }
}
