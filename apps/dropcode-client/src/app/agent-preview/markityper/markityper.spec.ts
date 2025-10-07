import { ComponentFixture, TestBed } from '@angular/core/testing';

import { Markityper } from './markityper';

describe('Markityper', () => {
  let component: Markityper;
  let fixture: ComponentFixture<Markityper>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Markityper]
    })
    .compileComponents();

    fixture = TestBed.createComponent(Markityper);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
