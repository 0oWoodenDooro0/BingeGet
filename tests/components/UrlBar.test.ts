import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import UrlBar from '../../src/components/UrlBar.vue';
import type { SearchResult } from '../../src/types';

describe('UrlBar.vue', () => {
  it('disables submit button when url is empty or only whitespace', async () => {
    const wrapper = mount(UrlBar, {
      props: {
        loading: false,
      },
    });

    // In direct URL mode
    const urlTab = wrapper.find('[data-testid="tab-url"]');
    if (urlTab.exists()) {
      await urlTab.trigger('click');
    }

    const button = wrapper.find('[data-testid="submit-btn"]');
    expect(button.attributes('disabled')).toBeDefined();

    const input = wrapper.find('[data-testid="url-input"]');
    await input.setValue('   ');
    expect(button.attributes('disabled')).toBeDefined();
  });

  it('emits parse event when valid url is submitted via click', async () => {
    const wrapper = mount(UrlBar, {
      props: {
        loading: false,
      },
    });

    const urlTab = wrapper.find('[data-testid="tab-url"]');
    if (urlTab.exists()) {
      await urlTab.trigger('click');
    }

    const input = wrapper.find('[data-testid="url-input"]');
    await input.setValue('https://chinaq.fun/tv-cn/12345/');

    const button = wrapper.find('[data-testid="submit-btn"]');
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

    const urlTab = wrapper.find('[data-testid="tab-url"]');
    if (urlTab.exists()) {
      await urlTab.trigger('click');
    }

    const input = wrapper.find('[data-testid="url-input"]');
    await input.setValue('https://chinaq.fun/tv-cn/12345/');

    const button = wrapper.find('[data-testid="submit-btn"]');
    expect(button.attributes('disabled')).toBeDefined();

    await input.trigger('keyup.enter');
    expect(wrapper.emitted('parse')).toBeFalsy();
  });

  it('disables search button when search keyword is empty in search mode', async () => {
    const wrapper = mount(UrlBar, {
      props: {
        loading: false,
      },
    });

    const searchTab = wrapper.find('[data-testid="tab-search"]');
    expect(searchTab.exists()).toBe(true);
    await searchTab.trigger('click');

    const searchBtn = wrapper.find('[data-testid="search-btn"]');
    expect(searchBtn.attributes('disabled')).toBeDefined();

    const searchInput = wrapper.find('[data-testid="search-input"]');
    await searchInput.setValue('   ');
    expect(searchBtn.attributes('disabled')).toBeDefined();
  });

  it('renders search results, emits select-result, and syncs url when a result is clicked', async () => {
    const mockResults: SearchResult[] = [
      {
        title: '繁花',
        year: '2023',
        category: '陸劇',
        url: 'https://chinaq.fun/tv-cn/202312271/',
        source_id: 'chinaq',
        source_name: 'ChinaQ',
      },
      {
        title: '繁星點點',
        year: null,
        category: '台劇',
        url: 'https://chinaq.fun/tv-tw/202209121/',
        source_id: 'chinaq',
        source_name: 'ChinaQ',
      },
    ];

    const searchHandler = vi.fn().mockResolvedValue(mockResults);

    const wrapper = mount(UrlBar, {
      props: {
        loading: false,
        onSearch: searchHandler,
      },
    });

    const searchTab = wrapper.find('[data-testid="tab-search"]');
    await searchTab.trigger('click');

    const searchInput = wrapper.find('[data-testid="search-input"]');
    await searchInput.setValue('繁');

    const searchBtn = wrapper.find('[data-testid="search-btn"]');
    await searchBtn.trigger('click');

    await vi.waitFor(() => {
      const items = wrapper.findAll('[data-testid="search-result-item"]');
      expect(items.length).toBe(2);
    });

    const firstItem = wrapper.findAll('[data-testid="search-result-item"]')[0];
    await firstItem.trigger('click');

    expect(wrapper.emitted('select-result')).toBeTruthy();
    expect(wrapper.emitted('select-result')![0]).toEqual([mockResults[0]]);

    // Switch to URL tab and verify url is synchronized
    const urlTab = wrapper.find('[data-testid="tab-url"]');
    await urlTab.trigger('click');
    const urlInput = wrapper.find<HTMLInputElement>('[data-testid="url-input"]');
    expect(urlInput.element.value).toBe(mockResults[0].url);
  });

  it('displays loading skeleton while search is in flight', async () => {
    let resolvePromise!: (results: SearchResult[]) => void;
    const searchPromise = new Promise<SearchResult[]>((resolve) => {
      resolvePromise = resolve;
    });

    const wrapper = mount(UrlBar, {
      props: {
        loading: false,
        onSearch: () => searchPromise,
      },
    });

    const searchTab = wrapper.find('[data-testid="tab-search"]');
    await searchTab.trigger('click');

    const searchInput = wrapper.find('[data-testid="search-input"]');
    await searchInput.setValue('繁花');

    const searchBtn = wrapper.find('[data-testid="search-btn"]');
    await searchBtn.trigger('click');

    // Skeleton should be visible
    expect(wrapper.find('[data-testid="search-skeleton"]').exists()).toBe(true);

    resolvePromise([]);
    await vi.waitFor(() => {
      expect(wrapper.find('[data-testid="search-skeleton"]').exists()).toBe(false);
    });
  });

  it('shows empty result indicator when search yields no results', async () => {
    const searchHandler = vi.fn().mockResolvedValue([]);

    const wrapper = mount(UrlBar, {
      props: {
        loading: false,
        onSearch: searchHandler,
      },
    });

    const searchTab = wrapper.find('[data-testid="tab-search"]');
    await searchTab.trigger('click');

    const searchInput = wrapper.find('[data-testid="search-input"]');
    await searchInput.setValue('不存在的劇名');

    const searchBtn = wrapper.find('[data-testid="search-btn"]');
    await searchBtn.trigger('click');

    await vi.waitFor(() => {
      const emptyState = wrapper.find('[data-testid="search-empty-state"]');
      expect(emptyState.exists()).toBe(true);
    });
  });

  it('does not have transition-all or focus:ring on input fields to prevent white border flicker/transition on focus', async () => {
    const wrapper = mount(UrlBar, {
      props: {
        loading: false,
      },
    });

    const searchInput = wrapper.find('[data-testid="search-input"]');
    expect(searchInput.classes()).not.toContain('transition-all');
    expect(searchInput.classes()).not.toContain('focus:ring-1');

    const urlTab = wrapper.find('[data-testid="tab-url"]');
    await urlTab.trigger('click');

    const urlInput = wrapper.find('[data-testid="url-input"]');
    expect(urlInput.classes()).not.toContain('transition-all');
    expect(urlInput.classes()).not.toContain('focus:ring-1');
  });
});
