import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { HttpClient } from '@angular/common/http';
import { RouterModule } from '@angular/router';

@Component({
  selector: 'app-waitlist',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterModule],
  templateUrl: './waitlist.component.html',
  styleUrls: ['./waitlist.component.scss']
})
export class WaitlistComponent {
  email = '';
  message = '';
  error = '';

  constructor(private http: HttpClient) {}

  joinWaitlist() {
    this.message = '';
    this.error = '';

    if (!this.email) {
      this.error = 'Please enter a valid email.';
      return;
    }

    this.http.post('/api/waitlist', { email: this.email }).subscribe({
      next: (res: any) => {
        this.message = res.message || 'You are on the waitlist!';
        this.email = '';
      },
      error: (err) => {
        if (err.status === 409) {
          this.message = "You're already on the list!";
        } else {
          this.error = 'An error occurred. Please try again.';
        }
      }
    });
  }
}