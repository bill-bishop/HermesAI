import { TestBed } from '@angular/core/testing';

import { ChatAnimationService } from './chat-animation-service';

describe('ChatAnimationService', () => {
  let service: ChatAnimationService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ChatAnimationService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
