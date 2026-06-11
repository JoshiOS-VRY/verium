import { describe, expect, it } from 'vitest';
import { checkpointFor, resumeStep, routeForProfile } from '@/lib/onboarding/intents';

describe('routeForProfile', () => {
  it('routes legacy upgrade through the dedicated full-node flow', () => {
    const route = routeForProfile({ intent: 'legacy_upgrade', mode: 'full_node' });
    expect(route.isLegacyUpgrade).toBe(true);
    expect(route.steps).toContain('hd_upgrade');
    expect(route.steps).toContain('recovery');
  });

  it('uses the short light flow when mode is light', () => {
    const route = routeForProfile({ intent: 'fresh_install', mode: 'light' });
    expect(route.isLegacyUpgrade).toBe(false);
    expect(route.steps).toEqual(['welcome', 'wallet', 'twofa', 'done']);
  });

  it('uses the full flow for a fresh full-node install', () => {
    const route = routeForProfile({ intent: 'fresh_install', mode: 'full_node' });
    expect(route.steps).toContain('daemon');
    expect(route.steps).toContain('bootstrap');
    expect(route.steps).not.toContain('hd_upgrade');
  });

  it('does not treat legacy upgrade as legacy when in light mode', () => {
    // A legacy wallet.dat cannot be opened by light mode; routing falls back to
    // the standard light flow (cross-mode migration is handled separately).
    const route = routeForProfile({ intent: 'legacy_upgrade', mode: 'light' });
    expect(route.isLegacyUpgrade).toBe(false);
  });
});

describe('checkpoint resume', () => {
  it('resumes an in-progress step that belongs to the route', () => {
    const route = routeForProfile({ intent: 'fresh_install', mode: 'full_node' });
    const cp = checkpointFor('fresh_install', 'recovery');
    expect(resumeStep(route, cp)).toBe('recovery');
  });

  it('falls back to the initial step for completed checkpoints', () => {
    const route = routeForProfile({ intent: 'fresh_install', mode: 'full_node' });
    const cp = checkpointFor('fresh_install', 'done');
    expect(cp.phase).toBe('complete');
    expect(resumeStep(route, cp)).toBe(route.initialStep);
  });

  it('ignores a step that is not part of the route', () => {
    const route = routeForProfile({ intent: 'fresh_install', mode: 'light' });
    const cp = checkpointFor('fresh_install', 'bootstrap');
    expect(resumeStep(route, cp)).toBe(route.initialStep);
  });
});
