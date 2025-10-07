import { ComponentFixture, TestBed } from '@angular/core/testing';

import { MarkityperComponent } from './markityper';

describe('Markityper', () => {
  let component: MarkityperComponent;
  let fixture: ComponentFixture<MarkityperComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [MarkityperComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(MarkityperComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
