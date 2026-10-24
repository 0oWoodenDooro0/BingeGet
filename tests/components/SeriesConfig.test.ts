import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import SeriesConfig from '../../src/components/SeriesConfig.vue';
import type { SeriesConfigModel } from '../../src/types';
import * as dialog from '@tauri-apps/plugin-dialog';

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

describe('SeriesConfig.vue', () => {
  const initialConfig: SeriesConfigModel = {
    customTitle: '繁花',
    season: 1,
    saveDir: '/home/user/Downloads',
    threads: 16,
    maxConcurrent: 2,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('triggers directory dialog and updates saveDir when browse button is clicked', async () => {
    const mockedOpen = vi.mocked(dialog.open);
    mockedOpen.mockResolvedValue('/home/user/Videos');

    const wrapper = mount(SeriesConfig, {
      props: {
        config: initialConfig,
        totalEpisodes: 30,
      },
    });

    const browseBtn = wrapper.find('button');
    await browseBtn.trigger('click');

    expect(mockedOpen).toHaveBeenCalledWith(
      expect.objectContaining({
        directory: true,
      })
    );

    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(wrapper.emitted('update:config')).toBeTruthy();
    const emittedConfig = wrapper.emitted('update:config')![0][0] as SeriesConfigModel;
    expect(emittedConfig.saveDir).toBe('/home/user/Videos');
  });

  it('does not update saveDir when directory selection is canceled', async () => {
    const mockedOpen = vi.mocked(dialog.open);
    mockedOpen.mockResolvedValue(null);

    const wrapper = mount(SeriesConfig, {
      props: {
        config: initialConfig,
        totalEpisodes: 30,
      },
    });

    const browseBtn = wrapper.find('button');
    await browseBtn.trigger('click');

    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(mockedOpen).toHaveBeenCalled();
    expect(wrapper.emitted('update:config')).toBeFalsy();
  });

  it('emits updated config when title input changes', async () => {
    const wrapper = mount(SeriesConfig, {
      props: {
        config: initialConfig,
        totalEpisodes: 30,
      },
    });

    const titleInput = wrapper.find('input[type="text"]');
    await titleInput.setValue('慶餘年');

    expect(wrapper.emitted('update:config')).toBeTruthy();
    const emittedConfig = wrapper.emitted('update:config')![0][0] as SeriesConfigModel;
    expect(emittedConfig.customTitle).toBe('慶餘年');
  });
});
