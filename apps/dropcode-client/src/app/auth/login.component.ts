import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { AuthService } from './auth.service';
import { ThirdPartyAuthComponent } from './third-party-auth/third-party-auth.component';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, RouterLink, ThirdPartyAuthComponent],
  templateUrl: './login.component.html'
})
export class LoginComponent {
  error: string | null = null;
  form;

  constructor(private fb: FormBuilder, private auth: AuthService) {
    this.form = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      password: ['', Validators.required],
    });
  }

  submit() {
    if (this.form.valid) {
      const { email, password } = this.form.value;
      this.auth.login({ email: email || '', password: password || '' }).subscribe({
        next: () => console.log('Login successful'),
        error: () => this.error = 'Login failed'
      });
    } else {
      this.error = 'Please enter valid credentials';
    }
  }
}