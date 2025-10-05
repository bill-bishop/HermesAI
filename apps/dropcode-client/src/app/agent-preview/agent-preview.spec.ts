import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AgentPreview } from './agent-preview';

describe('AgentPreview', () => {
  let component: AgentPreview;
  let fixture: ComponentFixture<AgentPreview>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AgentPreview]
    })
    .compileComponents();

    fixture = TestBed.createComponent(AgentPreview);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
