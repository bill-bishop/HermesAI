import { Routes, CanActivateFn, Router } from '@angular/router';
import { LoginComponent } from './auth/login.component';
import { RegisterComponent } from './auth/register.component';
import { inject } from '@angular/core';
import { AuthService } from './auth/auth.service';
import { HomeComponent } from './home/home.component';
import { FeaturesComponent } from './features/features.component';
import { PricingComponent } from './pricing/pricing.component';
import { filter, map } from 'rxjs/operators';

export const authGuard: CanActivateFn = () => {
  const auth = inject(AuthService);
  const router = inject(Router);

  return auth.currentUser$.pipe(
    filter(user => user !== undefined), // wait until loading finished
    map(user => {
      if (user) return true;
      return router.createUrlTree(['/login']);
    })
  );
};

export const routes: Routes = [
  { path: '', component: HomeComponent, canActivate: [authGuard] },
  { path: 'features', component: FeaturesComponent, canActivate: [authGuard] },
  { path: 'pricing', component: PricingComponent, canActivate: [authGuard] },
  { path: 'login', component: LoginComponent },
  { path: 'register', component: RegisterComponent },
];