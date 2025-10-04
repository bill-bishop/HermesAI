# Changelog

## [Unreleased]
- TBD

## [0.3.2] - 2025-10-02
### Added
- Support for `target` query parameter on login page to redirect users back to their original destination after GitHub OAuth login.
- `third-party-auth.component.ts` updated to use Angular `ActivatedRoute` for query param extraction.
- `third-party-auth.component.html` updated to bind `[href]` dynamically with `target`.

### Fixed
- GitHub login flow now correctly forwards `target` state through OAuth and back to the app.

## [0.3.1] - 2025-10-02
### Fixed
- Navbar hamburger toggle not working on mobile view due to missing Bootstrap JS import. Added `bootstrap.bundle.min.js` import in `main.ts` to enable collapse and dropdown functionality.

## [0.3.0] - 2025-10-02
### Added
- Login and Registration components with reactive forms and Bootstrap styling.
- GitHub login integration via `ThirdPartyAuthComponent`.
- Navbar updates with Login/Register links and authenticated user dropdown (avatar + logout).
- Routes for Home, Features, Pricing with placeholder Bootstrap-themed content.
- Auth guard integrated with `AuthService.currentUser$` observable.

### Fixed
- Incorrect use of `FormBuilder` initialization in Login/Register components.
- Resolved Angular 20 test runner mismatches (using `ng test` instead of Jest).
- Updated tests for `AppComponent` to check actual expectations (navbar + router-outlet).

## [0.2.0] - 2025-09-28
### Added
- Initial Angular 20 standalone app wiring.
- Jest + Karma test setup (deprecated in favor of Angular test bed).

## [0.1.0] - 2025-09-26
### Added
- Bootstrap theming integration.
- Angular monorepo initial scaffold under `apps/dropcode-client`. 