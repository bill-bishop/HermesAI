import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ReactiveFormsModule } from '@angular/forms';
import { RouterTestingModule } from '@angular/router/testing';
import { LoginComponent } from './login.component';
import { AuthService } from './auth.service';
import { of } from 'rxjs';

class MockAuthService {
  login() { return of(true); }
}

describe('LoginComponent', () => {
  let component: LoginComponent;
  let fixture: ComponentFixture<LoginComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ReactiveFormsModule, RouterTestingModule.withRoutes([]), LoginComponent],
      providers: [
        { provide: AuthService, useClass: MockAuthService }
      ]
    }).compileComponents();

    fixture = TestBed.createComponent(LoginComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should show error if form invalid', () => {
    component.form.controls['email'].setValue('');
    component.form.controls['password'].setValue('');
    component.submit();
    expect(component.error).toBeTruthy();
  });

  it('should call auth service if form valid', () => {
    const auth = TestBed.inject(AuthService);
    spyOn(auth, 'login').and.returnValue(of(true));
    component.form.controls['email'].setValue('test@test.com');
    component.form.controls['password'].setValue('123456');
    component.submit();
    expect(auth.login).toHaveBeenCalled();
  });
});