import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useSystemEnvironment, isTauri, getErrorMessage } from '../../src/composables/useSystemEnvironment';

describe('useSystemEnvironment composable', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    // Clean up window mock
    // @ts-expect-error Mock cleanup
    delete window.__TAURI_INTERNALS__;
  });

  it('reports non-tauri environment when __TAURI_INTERNALS__ is not present', () => {
    expect(isTauri()).toBe(false);
  });

  it('reports tauri environment when __TAURI_INTERNALS__ is present', () => {
    // @ts-expect-error Mock Tauri
    window.__TAURI_INTERNALS__ = {};
    expect(isTauri()).toBe(true);
  });

  it('correctly extracts error messages from various types', () => {
    expect(getErrorMessage(new Error('fail'))).toBe('fail');
    expect(getErrorMessage('string error')).toBe('string error');
    expect(getErrorMessage({ message: 'object message' })).toBe('object message');
    expect(getErrorMessage(123)).toBe('123');
  });

  it('provides mock environment in non-tauri mode', async () => {
    const { envInfo, checkEnvironment } = useSystemEnvironment({ autoCheck: false });
    expect(envInfo.value).toBeNull();

    const result = await checkEnvironment();
    expect(result.is_ready).toBe(true);
    expect(result.ffmpeg_path).toBe('/usr/bin/ffmpeg');
    expect(result.nm3u8dl_path).toBe('/usr/bin/n-m3u8dl-re');
    expect(result.gstreamer_ready).toBe(true);
    expect(envInfo.value).toEqual(result);
  });

  it('invokes get_system_environment in tauri environment', async () => {
    const mockTauriEnv = {
      nm3u8dl_path: '/custom/nm3u8dl-re',
      ffmpeg_path: '/custom/ffmpeg',
      is_ready: true,
      gstreamer_ready: true,
    };

    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: vi.fn().mockResolvedValue(mockTauriEnv),
    }));

    // @ts-expect-error Mock Tauri
    window.__TAURI_INTERNALS__ = {};

    const { envInfo, checkEnvironment } = useSystemEnvironment({ autoCheck: false });
    const result = await checkEnvironment();

    expect(result).toEqual(mockTauriEnv);
    expect(envInfo.value).toEqual(mockTauriEnv);
  });

  it('handles tauri invocation failures gracefully and sets is_ready false', async () => {
    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: vi.fn().mockRejectedValue(new Error('Backend error')),
    }));

    // @ts-expect-error Mock Tauri
    window.__TAURI_INTERNALS__ = {};

    const { envInfo, checkEnvironment } = useSystemEnvironment({ autoCheck: false });
    const result = await checkEnvironment();

    expect(result.is_ready).toBe(false);
    expect(result.ffmpeg_path).toBeNull();
    expect(result.nm3u8dl_path).toBeNull();
    expect(envInfo.value).toEqual(result);
  });
});
