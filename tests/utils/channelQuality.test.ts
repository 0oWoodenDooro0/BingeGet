import { describe, it, expect } from 'vitest';
import {
  parseResolutionProfile,
  getStreamQualityBadge,
} from '../../src/utils/channelQuality';

describe('channelQuality utils', () => {
  it('parses resolution tiers and score correctly', () => {
    const p4k = parseResolutionProfile('3840x2160');
    expect(p4k.label).toBe('4K');
    expect(p4k.tier).toBe('optimal');
    expect(p4k.score).toBeGreaterThan(5000);

    const p1080 = parseResolutionProfile('1920x1080 (1080p)');
    expect(p1080.label).toBe('1080p');
    expect(p1080.tier).toBe('optimal');

    const p720 = parseResolutionProfile('1280x720');
    expect(p720.label).toBe('720p');
    expect(p720.tier).toBe('good');

    const pUnknown = parseResolutionProfile('unknown');
    expect(pUnknown.tier).toBe('medium');
  });

  it('returns stream quality badge when custom probe details are present', () => {
    const badge = getStreamQualityBadge({
      resolution: '1920x1080',
      bitrate: '3,200 kbps',
    });

    expect(badge).not.toBeNull();
    expect(badge?.resolution).toBe('1080p');
    expect(badge?.bitrate).toBe('3,200 kbps');
    expect(badge?.tier).toBe('optimal');
  });

  it('returns null when no probed stream details are provided', () => {
    const badge = getStreamQualityBadge(undefined);
    expect(badge).toBeNull();
  });
});
