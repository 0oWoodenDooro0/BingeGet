import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import EpisodePicker from '../../src/components/EpisodePicker.vue';
import type { EpisodeInfo, SeriesMetadata, StreamDetails } from '../../src/types';

describe('EpisodePicker.vue', () => {
  const sampleEpisodes: EpisodeInfo[] = [
    { ep_number: 1, ep_title: '第1集', ep_id: 'ep1', is_downloaded: false },
    { ep_number: 2, ep_title: '第2集', ep_id: 'ep2', is_downloaded: false },
    { ep_number: 3, ep_title: '第3集', ep_id: 'ep3', is_downloaded: false },
  ];

  const sampleMetadata: SeriesMetadata = {
    title: '繁花',
    total_episodes: 30,
    cover_url: 'https://chinaq.fun/img/fan-hua.jpg',
    channels: [
      { id: 'ikzy', name: '愛坤 資源 (ikzy)' },
      { id: 'hnzy', name: '紅牛 資源 (hnzy)' },
    ],
  };

  const sampleProbeMap: Record<string, StreamDetails> = {
    ikzy: {
      resolution: '3840x2160 (4K)',
      bitrate: '8,500 kbps',
      video_codec: 'HEVC / H.265',
      audio_codec: 'E-AC-3',
      fps: 60,
      duration_secs: 3600,
    },
    hnzy: {
      resolution: '1920x1080 (1080p)',
      bitrate: '3,200 kbps',
      video_codec: 'H.264 / AVC',
      audio_codec: 'AAC',
      fps: 30,
      duration_secs: 2700,
    },
  };

  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('renders 7:5 deck layout with episode-deck on left (lg:col-span-7) and preview-dock on right (lg:col-span-5)', () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const grid = wrapper.find('.grid.grid-cols-1.lg\\:grid-cols-12.gap-5');
    expect(grid.exists()).toBe(true);

    const episodeDeck = wrapper.find('[data-testid="episode-deck"]');
    expect(episodeDeck.exists()).toBe(true);
    expect(episodeDeck.classes()).toContain('lg:col-span-7');

    const dock = wrapper.find('[data-testid="preview-dock"]');
    expect(dock.exists()).toBe(true);
    expect(dock.classes()).toContain('lg:col-span-5');

    const video = wrapper.find('[data-testid="preview-video"]');
    expect(video.exists()).toBe(true);

    const epItems = wrapper.findAll('[data-testid="episode-item"]');
    expect(epItems.length).toBe(3);
  });

  it('renders each episode card with a 3-zone structure (checkbox, title, preview button)', () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const epItems = wrapper.findAll('[data-testid="episode-item"]');
    expect(epItems.length).toBe(3);

    epItems.forEach((item) => {
      // Zone 1: Checkbox gutter
      const checkbox = item.find('[data-testid="episode-checkbox"]');
      expect(checkbox.exists()).toBe(true);

      // Zone 2: Dedicated episode title
      const title = item.find('[data-testid="episode-title"]');
      expect(title.exists()).toBe(true);

      // Zone 3: Dedicated preview button (testing both btn-preview-episode and legacy btn-preview-ep seam)
      const previewBtn = item.find('[data-testid="btn-preview-episode"]');
      expect(previewBtn.exists()).toBe(true);
      expect(item.find('[data-testid="btn-preview-ep"]').exists()).toBe(true);
    });

    // Checkbox reflects selected status
    const ep1Checkbox = wrapper.find('[data-episode-checkbox="1"]');
    expect(ep1Checkbox.attributes('data-checked')).toBe('true');

    const ep2Checkbox = wrapper.find('[data-episode-checkbox="2"]');
    expect(ep2Checkbox.attributes('data-checked')).toBe('false');

    // Preview button reflects active preview status (ep 1 initialized by default)
    const ep1PreviewBtn = wrapper.find('[data-episode-preview="1"]');
    expect(ep1PreviewBtn.attributes('data-active')).toBe('true');

    const ep2PreviewBtn = wrapper.find('[data-episode-preview="2"]');
    expect(ep2PreviewBtn.attributes('data-active')).toBe('false');
  });

  it('decouples preview switching from download selection: clicking preview button switches preview without mutating selectedEpisodes', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const ep2PreviewBtn = wrapper.find('[data-episode-preview="2"]');
    expect(ep2PreviewBtn.exists()).toBe(true);

    await ep2PreviewBtn.trigger('click');

    const changeEvents = wrapper.emitted('change-preview');
    expect(changeEvents).toBeTruthy();
    expect(changeEvents![changeEvents!.length - 1]).toEqual([2]);

    expect(wrapper.emitted('update:selectedEpisodes')).toBeFalsy();
    expect(ep2PreviewBtn.attributes('data-active')).toBe('true');
  });

  it('decouples download selection from preview switching: clicking checkbox toggles selectedEpisodes without switching preview', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const initialChangeEventsCount = (wrapper.emitted('change-preview') || []).length;

    const ep2Checkbox = wrapper.find('[data-episode-checkbox="2"]');
    await ep2Checkbox.trigger('click');

    const updateEvents = wrapper.emitted('update:selectedEpisodes');
    expect(updateEvents).toBeTruthy();
    expect(updateEvents![0]).toEqual([[1, 2]]);

    // change-preview must not be triggered by checkbox click
    const currentChangeEventsCount = (wrapper.emitted('change-preview') || []).length;
    expect(currentChangeEventsCount).toBe(initialChangeEventsCount);
  });

  it('supports batch selection toolbar placed inside episode-deck: select-all, clear-all, select-first-10, and select-first-20', async () => {
    const twentyFiveEpisodes: EpisodeInfo[] = Array.from({ length: 25 }, (_, i) => ({
      ep_number: i + 1,
      ep_title: `第${i + 1}集`,
      ep_id: `ep${i + 1}`,
      is_downloaded: false,
    }));

    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: twentyFiveEpisodes,
        selectedEpisodes: [1, 2],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    // Toolbar must be inside episode-deck (7 Columns deck)
    const episodeDeck = wrapper.find('[data-testid="episode-deck"]');
    expect(episodeDeck.exists()).toBe(true);

    const btnSelectAll = episodeDeck.find('[data-testid="btn-select-all"]');
    const btnClearAll = episodeDeck.find('[data-testid="btn-clear-all"]');
    const btnFirst10 = episodeDeck.find('[data-testid="btn-select-first-10"]');
    const btnFirst20 = episodeDeck.find('[data-testid="btn-select-first-20"]');

    expect(btnSelectAll.exists()).toBe(true);
    expect(btnClearAll.exists()).toBe(true);
    expect(btnFirst10.exists()).toBe(true);
    expect(btnFirst20.exists()).toBe(true);

    // Test Select All
    await btnSelectAll.trigger('click');
    const allExpected = Array.from({ length: 25 }, (_, i) => i + 1);
    expect(wrapper.emitted('update:selectedEpisodes')![0]).toEqual([allExpected]);

    // Test Clear All
    await btnClearAll.trigger('click');
    expect(wrapper.emitted('update:selectedEpisodes')![1]).toEqual([[]]);

    // Test First 10
    await btnFirst10.trigger('click');
    const first10Expected = Array.from({ length: 10 }, (_, i) => i + 1);
    expect(wrapper.emitted('update:selectedEpisodes')![2]).toEqual([first10Expected]);

    // Test First 20
    await btnFirst20.trigger('click');
    const first20Expected = Array.from({ length: 20 }, (_, i) => i + 1);
    expect(wrapper.emitted('update:selectedEpisodes')![3]).toEqual([first20Expected]);
  });

  it('supports custom range batch selection via inputs and apply-range button', async () => {
    const twentyFiveEpisodes: EpisodeInfo[] = Array.from({ length: 25 }, (_, i) => ({
      ep_number: i + 1,
      ep_title: `第${i + 1}集`,
      ep_id: `ep${i + 1}`,
      is_downloaded: false,
    }));

    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: twentyFiveEpisodes,
        selectedEpisodes: [],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    const startInput = wrapper.find('[data-testid="input-range-start"]');
    const endInput = wrapper.find('[data-testid="input-range-end"]');
    const applyBtn = wrapper.find('[data-testid="btn-apply-range"]');

    expect(startInput.exists()).toBe(true);
    expect(endInput.exists()).toBe(true);
    expect(applyBtn.exists()).toBe(true);

    await startInput.setValue('5');
    await endInput.setValue('10');
    await applyBtn.trigger('click');

    const expectedRange = [5, 6, 7, 8, 9, 10];
    expect(wrapper.emitted('update:selectedEpisodes')![0]).toEqual([expectedRange]);
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
    // Must have poster attribute set to cover_url
    expect(video.attributes('poster')).toBe(sampleMetadata.cover_url);

    await new Promise((r) => setTimeout(r, 20));
    await wrapper.vm.$nextTick();

    // Must NOT automatically invoke play() on mount
    expect(playSpy).not.toHaveBeenCalled();

    // Standby overlay is rendered before playback begins
    const standbyOverlay = wrapper.find('[data-testid="preview-standby-overlay"]');
    expect(standbyOverlay.exists()).toBe(true);
  });

  it('does not pulse the play icon during paused standby and only pulses when actively playing', async () => {
    vi.spyOn(HTMLMediaElement.prototype, 'play').mockImplementation(function (this: HTMLVideoElement) {
      if (this.onplay) {
        this.onplay(new Event('play'));
      }
      return Promise.resolve();
    });

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

    const ep1Btn = wrapper.find('[data-episode-preview="1"]');
    expect(ep1Btn.exists()).toBe(true);

    // During standby/paused state: Play icon must NOT have animate-pulse class
    const standbyPlayIcon = ep1Btn.find('svg');
    expect(standbyPlayIcon.exists()).toBe(true);
    expect(standbyPlayIcon.classes()).not.toContain('animate-pulse');

    // Trigger explicit playback
    const playPauseBtn = wrapper.find('[data-testid="btn-play-pause"]');
    await playPauseBtn.trigger('click');
    await wrapper.vm.$nextTick();

    // While playing: Active preview episode Play icon must have animate-pulse
    const playingPlayIcon = ep1Btn.find('svg');
    expect(playingPlayIcon.classes()).toContain('animate-pulse');
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

  it('triggers play() when clicking the standby overlay', async () => {
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

    const standbyOverlay = wrapper.find('[data-testid="preview-standby-overlay"]');
    expect(standbyOverlay.exists()).toBe(true);

    await standbyOverlay.trigger('click');
    expect(playSpy).toHaveBeenCalled();
  });

  it('guarantees 0-HUD video viewport with zero superimposed badges and retains standby overlay', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
        channelProbeMap: sampleProbeMap,
      },
    });

    await new Promise((r) => setTimeout(r, 10));
    await wrapper.vm.$nextTick();

    // 0-HUD requirement: The video box must NOT contain HUD overlay badges
    const hud = wrapper.find('[data-testid="preview-hud"]');
    expect(hud.exists()).toBe(false);

    const hudRes = wrapper.find('[data-testid="hud-resolution"]');
    expect(hudRes.exists()).toBe(false);

    const hudBitrate = wrapper.find('[data-testid="hud-bitrate"]');
    expect(hudBitrate.exists()).toBe(false);

    // Clean standby click-to-play overlay is retained
    const standbyOverlay = wrapper.find('[data-testid="preview-standby-overlay"]');
    expect(standbyOverlay.exists()).toBe(true);
  });

  it('renders dedicated telemetry dashboard below player controls with parameters from channelProbeMap', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
        channelProbeMap: sampleProbeMap,
      },
    });

    await new Promise((r) => setTimeout(r, 10));
    await wrapper.vm.$nextTick();

    // Telemetry dashboard card exists
    const telemetry = wrapper.find('[data-testid="telemetry-dashboard"]');
    expect(telemetry.exists()).toBe(true);

    // Displays verified telemetry parameters matching channelProbeMap
    const res = wrapper.find('[data-testid="telemetry-resolution"]');
    expect(res.exists()).toBe(true);
    expect(res.text()).toContain('3840x2160');

    const bitrate = wrapper.find('[data-testid="telemetry-bitrate"]');
    expect(bitrate.exists()).toBe(true);
    expect(bitrate.text()).toContain('8,500 kbps');

    const codecs = wrapper.find('[data-testid="telemetry-codecs"]');
    expect(codecs.exists()).toBe(true);
    expect(codecs.text()).toContain('HEVC');
    expect(codecs.text()).toContain('E-AC-3');

    const fpsDuration = wrapper.find('[data-testid="telemetry-fps-duration"]');
    expect(fpsDuration.exists()).toBe(true);
    expect(fpsDuration.text()).toContain('60 fps');

    const badge = wrapper.find('[data-testid="telemetry-status-badge"]');
    expect(badge.attributes('data-probed')).toBe('true');

    // Absolutely NO debug placeholders like "需 ffprobe"
    expect(res.text()).not.toContain('ffprobe');
    expect(bitrate.text()).not.toContain('ffprobe');
    expect(codecs.text()).not.toContain('ffprobe');
  });

  it('renders clean unprobed placeholders and unprobed badge when channelProbeMap is empty or missing', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
        channelProbeMap: {},
      },
    });

    await new Promise((r) => setTimeout(r, 10));
    await wrapper.vm.$nextTick();

    const telemetry = wrapper.find('[data-testid="telemetry-dashboard"]');
    expect(telemetry.exists()).toBe(true);

    const res = wrapper.find('[data-testid="telemetry-resolution"]');
    expect(res.text()).toBe('--');

    const bitrate = wrapper.find('[data-testid="telemetry-bitrate"]');
    expect(bitrate.text()).toBe('--');

    const codecs = wrapper.find('[data-testid="telemetry-codecs"]');
    expect(codecs.text()).toBe('--');

    const fpsDuration = wrapper.find('[data-testid="telemetry-fps-duration"]');
    expect(fpsDuration.text()).toBe('--');

    const badge = wrapper.find('[data-testid="telemetry-status-badge"]');
    expect(badge.attributes('data-probed')).toBe('false');
  });

  it('updates telemetry dashboard immediately when selectedChannel switches', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
        channelProbeMap: sampleProbeMap,
      },
    });

    await new Promise((r) => setTimeout(r, 10));
    await wrapper.vm.$nextTick();

    expect(wrapper.find('[data-testid="telemetry-resolution"]').text()).toContain('3840x2160');

    // Switch to hnzy channel
    await wrapper.setProps({
      selectedChannel: 'hnzy',
    });
    await new Promise((r) => setTimeout(r, 10));
    await wrapper.vm.$nextTick();

    expect(wrapper.find('[data-testid="telemetry-resolution"]').text()).toContain('1080p');
    expect(wrapper.find('[data-testid="telemetry-bitrate"]').text()).toContain('3,200 kbps');
  });

  it('refines player controls: removes download toggle button and applies dynamic gradient fill to scrubber and volume with full seam compatibility', async () => {
    const wrapper = mount(EpisodePicker, {
      props: {
        episodes: sampleEpisodes,
        selectedEpisodes: [1],
        metadata: sampleMetadata,
        selectedChannel: 'ikzy',
      },
    });

    // 1. Redundant download queue toggle button must NOT exist in the player controls
    const downloadToggleBtn = wrapper.find('[data-testid="btn-toggle-download-current"]');
    expect(downloadToggleBtn.exists()).toBe(false);

    // 2. Play/Pause and Mute buttons exist
    const playPauseBtn = wrapper.find('[data-testid="btn-play-pause"]');
    expect(playPauseBtn.exists()).toBe(true);

    const muteBtn = wrapper.find('[data-testid="btn-mute"]');
    expect(muteBtn.exists()).toBe(true);

    // 3. Scrubber with dynamic gradient fill (compatible with preview-scrubber and timeline-scrubber seams)
    const scrubber = wrapper.find('[data-testid="preview-scrubber"]');
    expect(scrubber.exists()).toBe(true);
    expect(scrubber.attributes('style')).toContain('linear-gradient');
    expect(scrubber.attributes('style')).toContain('#8b5cf6');
    expect(scrubber.attributes('style')).toContain('#334155');

    const timelineScrubber = wrapper.find('[data-testid="timeline-scrubber"]');
    expect(timelineScrubber.exists()).toBe(true);

    // 4. Volume slider with dynamic gradient fill (compatible with preview-volume and volume-slider seams)
    const volumeSlider = wrapper.find('[data-testid="preview-volume"]');
    expect(volumeSlider.exists()).toBe(true);
    expect(volumeSlider.attributes('style')).toContain('linear-gradient');
    expect(volumeSlider.attributes('style')).toContain('#8b5cf6');
    expect(volumeSlider.attributes('style')).toContain('#334155');

    const volumeSliderLegacy = wrapper.find('[data-testid="volume-slider"]');
    expect(volumeSliderLegacy.exists()).toBe(true);

    // 5. Single-click Copy Stream URL button in player control bar
    const copyBtn = wrapper.find('[data-testid="btn-copy-stream-url"]');
    expect(copyBtn.exists()).toBe(true);

    await copyBtn.trigger('click');
    expect(copyBtn.exists()).toBe(true);
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

    const retryBtn = wrapper.find('[data-testid="btn-retry-preview"]');
    expect(retryBtn.exists()).toBe(true);

    await retryBtn.trigger('click');
    const changeEvents = wrapper.emitted('change-preview');
    expect(changeEvents).toBeTruthy();
  });
});
