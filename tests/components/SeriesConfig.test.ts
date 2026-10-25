import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
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

    const browseBtn = wrapper.find('[data-testid=\"browse-dir-btn\"]');
    await browseBtn.trigger('click');
    await flushPromises();

    expect(mockedOpen).toHaveBeenCalledWith(
      expect.objectContaining({
        directory: true,
      })
    );

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

    const browseBtn = wrapper.find('[data-testid=\"browse-dir-btn\"]');
    await browseBtn.trigger('click');
    await flushPromises();

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

    const titleInput = wrapper.find('[data-testid=\"series-title-input\"]');
    await titleInput.setValue('慶餘年');

    expect(wrapper.emitted('update:config')).toBeTruthy();
    const emitted = wrapper.emitted('update:config')![0][0] as SeriesConfigModel;
    expect(emitted.customTitle).toBe('慶餘年');
  });

  it('renders series cover artwork when coverUrl is provided', () => {
    const wrapper = mount(SeriesConfig, {
      props: {
        config: initialConfig,
        totalEpisodes: 30,
        coverUrl: 'https://chinaq.fun/img/202614235.jpg',
      },
    });

    const coverBox = wrapper.find('[data-testid=\"series-header-cover\"]');
    expect(coverBox.exists()).toBe(true);
    const img = coverBox.find('img');
    expect(img.exists()).toBe(true);
    expect(img.attributes('src')).toBe('https://chinaq.fun/img/202614235.jpg');
  });
});
