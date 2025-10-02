import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ReactiveFormsModule } from '@angular/forms';
import { RouterTestingModule } from '@angular/router/testing';
import { RegisterComponent } from './register.component';
import { AuthService } from './auth.service';
import { of } from 'rxjs';

class MockAuthService {
  register() { return of(true); }
}

describe('RegisterComponent', () => {
  let component: RegisterComponent;
  let fixture: ComponentFixture<RegisterComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ReactiveFormsModule, RouterTestingModule.withRoutes([]), RegisterComponent],
      providers: [
        { provide: AuthService, useClass: MockAuthService }
      ]
    }).compileComponents();

    fixture = TestBed.createComponent(RegisterComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should show error if passwords do not match', () => {
    component.form.controls['email'].setValue('test@test.com');
    component.form.controls['password'].setValue('123456');
    component.form.controls['confirmPassword'].setValue('654321');
    component.submit();
    expect(component.error).toBe('Passwords do not match');
  });

  it('should call auth service if form valid', () => {
    const auth = TestBed.inject(AuthService);
    spyOn(auth, 'register').and.returnValue(of(true));
    component.form.controls['email'].setValue('test@test.com');
    component.form.controls['password'].setValue('123456');
    component.form.controls['confirmPassword'].setValue('123456');
    component.submit();
    expect(auth.register).toHaveBeenCalled();
  });
});