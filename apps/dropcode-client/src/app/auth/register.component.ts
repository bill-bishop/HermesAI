import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { AuthService } from './auth.service';
import { ThirdPartyAuthComponent } from './third-party-auth/third-party-auth.component';

@Component({
  selector: 'app-register',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, RouterLink, ThirdPartyAuthComponent],
  templateUrl: './register.component.html'
})
export class RegisterComponent {
  error: string | null = null;
  form;

  constructor(private fb: FormBuilder, private auth: AuthService) {
    this.form = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      password: ['', Validators.required],
      confirmPassword: ['', Validators.required],
    });
  }

  submit() {
    if (this.form.valid) {
      const { email, password, confirmPassword } = this.form.value;
      if (password !== confirmPassword) {
        this.error = 'Passwords do not match';
        return;
      }
      this.auth.register({
        email: email || '',
        password: password || '',
        confirmPassword: confirmPassword || ''
      }).subscribe({
        next: () => console.log('Registration successful'),
        error: () => this.error = 'Registration failed'
      });
    } else {
      this.error = 'Please fill out the form correctly';
    }
  }
}