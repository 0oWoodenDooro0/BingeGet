import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import EpisodePicker from '../../src/components/EpisodePicker.vue';
import type { EpisodeInfo, SeriesMetadata } from '../../src/types';

describe('EpisodePicker.vue', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  const sampleEpisodes: EpisodeInfo[] = [
    { ep_number: 1, ep_id: 'ep1', available_channels: ['ikzy', 'hnzy'] },
    { ep_number: 2, ep_id: 'ep2', available_channels: ['ikzy', 'hnzy'] },
    { ep_number: 3, ep_id: 'ep3', available_channels: ['ikzy', 'hnzy'] },
  ];

  const sampleMetadata: SeriesMetadata = {
    drama_id: '202614235',
    category_slug: 'tv-cn',
    cover_url: 'https://chinaq.fun/img/202614235.jpg',
    episodes: sampleEpisodes,
    channels: [
      { id: 'ikzy', name: 'iKun 資源 (ikzy) - 推薦' },
      { id: 'hnzy', name: '紅牛 資源 (hnzy)' },
    ],
  };

  it('renders a 2-column layout with episode list on the left and preview dock on the right', () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1, 2],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const dock = wrapper.find('[data-testid="preview-dock"]');
    expect(dock.exists()).toBe(true);

    const video = wrapper.find('[data-testid="preview-video"]');
    expect(video.exists()).toBe(true);

    const epItems = wrapper.findAll('[data-testid="episode-item"]');
    expect(epItems.length).toBe(3);
  });

  it('disables autoplay by default on mount and does not include autoplay attribute on video', async () => {
    const playSpy = vi.spyOn(HTMLMediaElement.prototype, 'play').mockResolvedValue(undefined);

    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const video = wrapper.find('[data-testid="preview-video"]');
    expect(video.exists()).toBe(true);
    // Must NOT have autoplay HTML attribute
    expect(video.attributes('autoplay')).toBeUndefined();

    await new Promise((r) => setTimeout(r, 20));
    await wrapper.vm.$nextTick();

    // Must NOT automatically invoke play() on mount
    expect(playSpy).not.toHaveBeenCalled();
  });

  it('triggers play() when the user explicitly clicks the play button', async () => {
    const playSpy = vi.spyOn(HTMLMediaElement.prototype, 'play').mockResolvedValue(undefined);

    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    await new Promise((r) => setTimeout(r, 20));
    await wrapper.vm.$nextTick();

    const playPauseBtn = wrapper.find('[data-testid="btn-play-pause"]');
    expect(playPauseBtn.exists()).toBe(true);

    await playPauseBtn.trigger('click');
    expect(playSpy).toHaveBeenCalled();
  });

  it('decouples preview switching from download selection: clicking item switches preview without mutating selectedEpisodes', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const ep2Item = wrapper.find('[data-episode-number="2"]');
    await ep2Item.trigger('click');

    const changeEvents = wrapper.emitted('change-preview');
    expect(changeEvents).toBeTruthy();
    expect(changeEvents![changeEvents!.length - 1]).toEqual([2]);

    expect(wrapper.emitted('update:selectedEpisodes')).toBeFalsy();
    expect(ep2Item.classes()).toContain('border-violet-500');
  });

  it('toggles download selection when clicking dedicated checkbox without switching preview', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const ep2Checkbox = wrapper.find('[data-episode-checkbox="2"]');
    await ep2Checkbox.trigger('click');

    expect(wrapper.emitted('update:selectedEpisodes')).toBeTruthy();
    expect(wrapper.emitted('update:selectedEpisodes')![0]).toEqual([[1, 2]]);
  });

  it('renders stream details HUD with resolution and bitrate once probed', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    await new Promise((r) => setTimeout(r, 10));
    await wrapper.vm.$nextTick();

    const hud = wrapper.find('[data-testid="preview-hud"]');
    expect(hud.exists()).toBe(true);

    const resBadge = wrapper.find('[data-testid="hud-resolution"]');
    expect(resBadge.exists()).toBe(true);

    const bitrateBadge = wrapper.find('[data-testid="hud-bitrate"]');
    expect(bitrateBadge.exists()).toBe(true);
  });

  it('reloads preview stream and details when selectedChannel changes', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    await new Promise((r) => setTimeout(r, 10));

    // Change channel prop
    await wrapper.setProps({
      selectedChannel: 'hnzy',
    });

    await new Promise((r) => setTimeout(r, 10));
    await wrapper.vm.$nextTick();

    const channelIndicator = wrapper.find('[data-testid="channel-indicator"]');
    expect(channelIndicator.attributes('title')).toBe('紅牛 資源 (hnzy)');
  });

  it('renders error overlay with retry button when playback fails and allows retrying', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    await new Promise((r) => setTimeout(r, 10));
    await wrapper.vm.$nextTick();

    const videoEl = wrapper.find<HTMLVideoElement>('[data-testid="preview-video"]').element;

    // Simulate error event on video
    Object.defineProperty(videoEl, 'error', {
      value: {
        code: 3,
        message: 'GStreamer decode error',
      },
      configurable: true,
    });
    if (videoEl.onerror) {
      videoEl.onerror(new Event('error'));
    }

    await wrapper.vm.$nextTick();

    const errorOverlay = wrapper.find('[data-testid="preview-error-overlay"]');
    expect(errorOverlay.exists()).toBe(true);
    expect(errorOverlay.text()).toMatch(/解碼|GStreamer/i);

    const retryBtn = wrapper.find('[data-testid="btn-retry-preview"]');
    expect(retryBtn.exists()).toBe(true);

    await retryBtn.trigger('click');
    const changeEvents = wrapper.emitted('change-preview');
    expect(changeEvents).toBeTruthy();
  });

  it('provides play/pause and mute/unmute buttons, timeline scrubber, and volume slider', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const playPauseBtn = wrapper.find('[data-testid="btn-play-pause"]');
    expect(playPauseBtn.exists()).toBe(true);

    const muteBtn = wrapper.find('[data-testid="btn-mute"]');
    expect(muteBtn.exists()).toBe(true);

    const timelineScrubber = wrapper.find('[data-testid="timeline-scrubber"]');
    expect(timelineScrubber.exists()).toBe(true);

    const volumeSlider = wrapper.find('[data-testid="volume-slider"]');
    expect(volumeSlider.exists()).toBe(true);
    expect(volumeSlider.attributes('type')).toBe('range');
  });
});
