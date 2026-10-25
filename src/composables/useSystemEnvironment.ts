import { ref, onMounted, getCurrentInstance } from 'vue';
import type { SystemEnvironment } from '../types';

export const isTauri = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

export const getErrorMessage = (err: unknown): string => {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;
  if (err && typeof err === 'object' && 'message' in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
};

export interface UseSystemEnvironmentOptions {
  autoCheck?: boolean;
}

export function useSystemEnvironment(options: UseSystemEnvironmentOptions = {}) {
  const { autoCheck = true } = options;
  const envInfo = ref<SystemEnvironment | null>(null);

  const checkEnvironment = async (): Promise<SystemEnvironment> => {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const info = await invoke<SystemEnvironment>('get_system_environment');
        envInfo.value = info;
        return info;
      } catch (err: unknown) {
        console.warn('Tauri initialization note:', getErrorMessage(err));
        const fallback: SystemEnvironment = {
          nm3u8dl_path: null,
          ffmpeg_path: null,
          is_ready: false,
          gstreamer_ready: undefined,
        };
        envInfo.value = fallback;
        return fallback;
      }
    } else {
      const mockEnv: SystemEnvironment = {
        nm3u8dl_path: '/usr/bin/n-m3u8dl-re',
        ffmpeg_path: '/usr/bin/ffmpeg',
        is_ready: true,
        gstreamer_ready: true,
      };
      envInfo.value = mockEnv;
      return mockEnv;
    }
  };

  if (autoCheck && getCurrentInstance()) {
    onMounted(async () => {
      await checkEnvironment();
    });
  }

  return {
    envInfo,
    checkEnvironment,
  };
}
