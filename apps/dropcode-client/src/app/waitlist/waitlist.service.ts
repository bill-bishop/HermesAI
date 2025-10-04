import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, throwError } from 'rxjs';
import { catchError } from 'rxjs/operators';

@Injectable({ providedIn: 'root' })
export class WaitlistService {
  private apiUrl = '/api/waitlist';

  constructor(private http: HttpClient) {}

  joinWaitlist(email: string, source?: string): Observable<any> {
    return this.http.post(this.apiUrl, { email, source }).pipe(
      catchError((error) => {
        console.error('WaitlistService error:', error);
        return throwError(() => error);
      })
    );
  }
}