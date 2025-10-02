import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { BehaviorSubject, Observable } from 'rxjs';
import { tap } from 'rxjs/operators';

export interface User {
  id: number;
  email: string;
  login?: string;
  avatarUrl?: string;
}

@Injectable({ providedIn: 'root' })
export class AuthService {
  private currentUserSubject = new BehaviorSubject<User | null | undefined>(undefined);
  public currentUser$ = this.currentUserSubject.asObservable();

  constructor(private http: HttpClient) {
    // On init, try to fetch current user
    this.me().subscribe({
      next: (user) => this.currentUserSubject.next(user),
      error: () => this.currentUserSubject.next(null),
    });
  }

  login(credentials: { email: string; password: string }) {
    return this.http.post<User>('/api/auth/login', credentials, { withCredentials: true }).pipe(
      tap((user) => this.currentUserSubject.next(user))
    );
  }

  register(credentials: { email: string; password: string; confirmPassword: string }) {
    return this.http.post<User>('/api/auth/register', credentials, { withCredentials: true }).pipe(
      tap((user) => this.currentUserSubject.next(user))
    );
  }

  me(): Observable<User> {
    return this.http.get<User>('/api/auth/me', { withCredentials: true });
  }

  logout() {
    return this.http.post('/api/auth/logout', {}, { withCredentials: true }).pipe(
      tap(() => this.currentUserSubject.next(null))
    );
  }
}