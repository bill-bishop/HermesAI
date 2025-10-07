import {Component, Input, OnInit} from '@angular/core';
import {ChatAnimationService} from '../chat-animation-service';

@Component({
  selector: 'app-thinking',
  imports: [],
  templateUrl: './thinking.html',
  styleUrl: './thinking.scss'
})
export class Thinking implements OnInit {


  @Input()
  message: string = '';

  @Input()
  delay: number = 3000;

  constructor(private chatAnimation: ChatAnimationService) {}

  ngOnInit() {
    setTimeout(() => {
      this.chatAnimation.notifyDone();
    }, this.delay);
  }
}
