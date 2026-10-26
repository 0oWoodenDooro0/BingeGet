import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ChannelSelector from '../../src/components/ChannelSelector.vue';
import type { ChannelOption } from '../../src/types';
import { useSeriesSession } from '../../src/composables/useSeriesSession';

describe('ChannelSelector.vue', () => {
  const sampleChannels: ChannelOption[] = [
    { id: 'jszy', name: 'jszy' },
    { id: 'ikzy', name: 'ikzy' },
    { id: 'hnzy', name: 'hnzy' },
  ];

  it('renders channels in original order without sorting by quality', () => {
    const wrapper = mount(ChannelSelector, {
      props: {
        channels: sampleChannels,
        selectedChannel: 'ikzy',
      },
    });

    const options = wrapper.findAll('[data-testid="channel-item"]');
    expect(options.length).toBe(3);
    // Preserves original array order: jszy, then ikzy, then hnzy
    expect(options[0].attributes('data-channel-id')).toBe('jszy');
    expect(options[1].attributes('data-channel-id')).toBe('ikzy');
    expect(options[2].attributes('data-channel-id')).toBe('hnzy');
  });

  it('emits update:selectedChannel when a channel is clicked', async () => {
    const wrapper = mount(ChannelSelector, {
      props: {
        channels: sampleChannels,
        selectedChannel: 'ikzy',
      },
    });

    const hnzyOption = wrapper.find('[data-channel-id="hnzy"]');
    await hnzyOption.trigger('click');

    expect(wrapper.emitted('update:selectedChannel')).toBeTruthy();
    expect(wrapper.emitted('update:selectedChannel')![0]).toEqual(['hnzy']);
  });

  it('does not display quality badges or bitrate on channel buttons', () => {
    const wrapper = mount(ChannelSelector, {
      props: {
        channels: sampleChannels,
        selectedChannel: 'ikzy',
      },
    });

    expect(wrapper.find('[data-testid="channel-quality-badge"]').exists()).toBe(false);
    const ikzyBtn = wrapper.find('[data-channel-id="ikzy"]');
    expect(ikzyBtn.text()).toBe('ikzy');
  });

  it('renders clean summary counter and does not render sorting subtitle or priority badge in header', () => {
    const wrapper = mount(ChannelSelector, {
      props: {
        channels: sampleChannels,
        selectedChannel: 'ikzy',
      },
    });

    const counter = wrapper.find('[data-testid="channel-summary-counter"]');
    expect(counter.exists()).toBe(true);
    expect(counter.text()).toContain(String(sampleChannels.length));

    expect(wrapper.find('[data-testid="channel-quality-priority-badge"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="channel-sort-subtitle"]').exists()).toBe(false);
  });

  it('binds directly to SeriesSession workspace and updates selectedChannel on click', async () => {
    const session = useSeriesSession();
    session.metadata.value = {
      drama_id: '999',
      channels: sampleChannels,
      episodes: [],
    };
    session.selectChannel('ikzy');

    const wrapper = mount(ChannelSelector, {
      props: {
        session,
      },
    });

    const options = wrapper.findAll('[data-testid="channel-item"]');
    expect(options.length).toBe(3);

    const hnzyBtn = wrapper.find('[data-channel-id="hnzy"]');
    await hnzyBtn.trigger('click');

    expect(session.selectedChannel.value).toBe('hnzy');
    expect(hnzyBtn.classes()).toContain('border-indigo-500');
  });
});
