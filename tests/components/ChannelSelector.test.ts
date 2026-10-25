import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ChannelSelector from '../../src/components/ChannelSelector.vue';
import type { ChannelOption } from '../../src/types';

describe('ChannelSelector.vue', () => {
  const sampleChannels: ChannelOption[] = [
    { id: 'jszy', name: '極速 資源 (jszy)' },
    { id: 'ikzy', name: 'iKun 資源 (ikzy)' },
    { id: 'hnzy', name: '紅牛 資源 (hnzy)' },
  ];

  it('renders channels sorted by quality from highest to lowest by default', () => {
    const wrapper = mount(ChannelSelector, {
      props: {
        channels: sampleChannels,
        selectedChannel: 'ikzy',
      },
    });

    const options = wrapper.findAll('[data-testid="channel-item"]');
    expect(options.length).toBe(3);
    // ikzy (highest) should be first, then hnzy, then jszy
    expect(options[0].attributes('data-channel-id')).toBe('ikzy');
    expect(options[1].attributes('data-channel-id')).toBe('hnzy');
    expect(options[2].attributes('data-channel-id')).toBe('jszy');
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

  it('displays quality tags on each channel card', () => {
    const wrapper = mount(ChannelSelector, {
      props: {
        channels: sampleChannels,
        selectedChannel: 'ikzy',
      },
    });

    const ikzyBadge = wrapper.find('[data-channel-id="ikzy"] [data-testid="channel-quality-badge"]');
    expect(ikzyBadge.exists()).toBe(true);
    expect(ikzyBadge.text()).toContain('1080p');
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
    expect(wrapper.html().includes('依畫質高至低排序')).toBe(false);
    expect(wrapper.html().includes('高畫質優先')).toBe(false);
  });
});
