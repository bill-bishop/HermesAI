import {Component, Input} from '@angular/core';
import {ChatAnimationService} from '../chat-animation-service';
import {MarkityperComponent} from '../markityper/markityper';

@Component({
  selector: 'app-chat-message',
  imports: [MarkityperComponent],
  templateUrl: './chat-message.html',
  styleUrl: './chat-message.scss'
})
export class ChatMessage {
  @Input()
  type: string = 'user';

  @Input()
  meta: string = '';

  @Input()
  style: string = 'text typing';

  @Input()
  message: string = '';

  chatAnimation: ChatAnimationService;

  constructor(chatAnimation: ChatAnimationService) {
    this.chatAnimation = chatAnimation;
  }

  avatarLetter(): string {
    return `${this.type || ' '}`.charAt(0).toUpperCase();
  }
}
