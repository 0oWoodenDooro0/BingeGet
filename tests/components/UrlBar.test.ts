import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import UrlBar from '../../src/components/UrlBar.vue';

describe('UrlBar.vue', () => {
  it('disables submit button when url is empty or only whitespace', async () => {
    const wrapper = mount(UrlBar, {
      props: {
        loading: false,
      },
    });

    const button = wrapper.find('button');
    expect(button.attributes('disabled')).toBeDefined();

    const input = wrapper.find('input');
    await input.setValue('   ');
    expect(button.attributes('disabled')).toBeDefined();
  });

  it('emits parse event when valid url is submitted via click', async () => {
    const wrapper = mount(UrlBar, {
      props: {
        loading: false,
      },
    });

    const input = wrapper.find('input');
    await input.setValue('https://chinaq.fun/tv-cn/12345/');

    const button = wrapper.find('button');
    expect(button.attributes('disabled')).toBeUndefined();

    await button.trigger('click');
    expect(wrapper.emitted('parse')).toBeTruthy();
    expect(wrapper.emitted('parse')![0]).toEqual(['https://chinaq.fun/tv-cn/12345/']);
  });

  it('does not emit parse event when loading is true', async () => {
    const wrapper = mount(UrlBar, {
      props: {
        loading: true,
      },
    });

    const input = wrapper.find('input');
    await input.setValue('https://chinaq.fun/tv-cn/12345/');

    const button = wrapper.find('button');
    expect(button.attributes('disabled')).toBeDefined();

    await input.trigger('keyup.enter');
    expect(wrapper.emitted('parse')).toBeFalsy();
  });
});
