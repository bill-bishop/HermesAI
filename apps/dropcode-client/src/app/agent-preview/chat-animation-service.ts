import { Injectable } from '@angular/core';
import { Subject } from 'rxjs';

@Injectable({
  providedIn: 'root'
})
export class ChatAnimationService {
  private _done = new Subject<{ id: string }>();
  done$ = this._done.asObservable();

  notifyDone() {
    this._done.next({ id: "success" });
  }
}
