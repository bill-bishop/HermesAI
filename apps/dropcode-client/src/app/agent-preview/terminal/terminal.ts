import {Component, OnInit} from '@angular/core';
import {ChatAnimationService} from '../chat-animation-service';

@Component({
  selector: 'app-terminal',
  imports: [],
  templateUrl: './terminal.html',
  styleUrl: './terminal.scss'
})
export class Terminal implements OnInit {

  constructor(private chatAnimation: ChatAnimationService) {
  }

  ngOnInit() {
    setTimeout(() => {
      this.chatAnimation.notifyDone();
    }, 7000);
  }
}
