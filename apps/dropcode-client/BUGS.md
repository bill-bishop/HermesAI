# HermesAI Client – Debug Retrospective

This document catalogs the changes that failed during implementation or testing.

---

## Failed Changes

1. **Test Runner Confusion (Jest vs Angular/Karma)**
   - Initial errors with `describe/it/expect` not found.
   - Multiple attempts to wire Jest (`jest-preset-angular`, `setup-jest.ts`).
   - Ultimately, only `ng test` (Karma) ran the TestBed properly; Jest never worked.

2. **Component Template Resolution Errors**
   - Errors: “Component not resolved” with `templateUrl`/`styleUrls`.
   - Tried inline template hacks, fiddling with `tsconfig.include`.
   - Actual fix came from proper Angular test setup.

3. **Circular Dependency / Missing Provider Issues**
   - Runtime `NG0200` circular dependency for `AuthService`.
   - Later: `No provider for ActivatedRoute` in tests.

4. **Running Tests in Docker**
   - Chromium failed with `Running as root without --no-sandbox`.
   - CLI flags attempt (`ng test -- --no-sandbox`) rejected.
   - Required custom launcher in `karma.conf.js`.

5. **Standalone Component Imports**
   - `app-third-party-auth` unknown element error.
   - Missing from `imports` array of `LoginComponent` and `RegisterComponent`.

---

✅ End of list. Further analysis to be done manually.
