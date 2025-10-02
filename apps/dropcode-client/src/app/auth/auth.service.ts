import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { BehaviorSubject, Observable } from 'rxjs';
import { tap } from 'rxjs/operators';

export interface User {
  id: number;
  email: string;
  provider?: string;
  provider_id?: string;
  login?: string;        // GitHub username
  avatar_url?: string;   // GitHub avatar
}

@Injectable({ providedIn: 'root' })
export class AuthService {
  private currentUserSubject = new BehaviorSubject<User | null>(null);
  public currentUser$ = this.currentUserSubject.asObservable();

  constructor(private http: HttpClient) {
    // Try to load user on startup
    this.me().subscribe({
      next: (user) => this.currentUserSubject.next(user),
      error: () => this.currentUserSubject.next(null)
    });
  }

  login(credentials: { email: string; password: string }): Observable<any> {
    return this.http.post<User>('/api/auth/login', credentials, { withCredentials: true }).pipe(
      tap((user) => this.currentUserSubject.next(user))
    );
  }

  register(credentials: { email: string; password: string; confirmPassword: string }): Observable<any> {
    return this.http.post<User>('/api/auth/register', credentials, { withCredentials: true }).pipe(
      tap((user) => this.currentUserSubject.next(user))
    );
  }

  me(): Observable<User> {
    return this.http.get<User>('/api/auth/me', {
      headers: { Authorization: `Bearer ${localStorage.getItem('auth_token')}` }
    });
  }

  logout(): Observable<any> {
    return this.http.post('/api/auth/logout', {}, { withCredentials: true }).pipe(
      tap(() => this.currentUserSubject.next(null))
    );
  }

  isAuthenticated(): boolean {
    return this.currentUserSubject.value !== null;
  }

  getCurrentUser(): User | null {
    return this.currentUserSubject.value;
  }
}
