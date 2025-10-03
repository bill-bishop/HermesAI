import { Component } from '@angular/core';
import { HttpClient } from '@angular/common/http';

@Component({
  selector: 'app-canvas',
  template: `<iframe [srcdoc]="html" style="width:100%;height:100vh;border:none;"></iframe>`
})
export class CanvasComponent {
  html = '';
  constructor(private http: HttpClient) {
    this.http.get('/api/canvas', { responseType: 'text' }).subscribe(res => {
      this.html = res;
    });
  }
}