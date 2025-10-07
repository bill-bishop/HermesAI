import { ComponentFixture, TestBed } from '@angular/core/testing';

import { Thinking } from './thinking';

describe('Thinking', () => {
  let component: Thinking;
  let fixture: ComponentFixture<Thinking>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Thinking]
    })
    .compileComponents();

    fixture = TestBed.createComponent(Thinking);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
