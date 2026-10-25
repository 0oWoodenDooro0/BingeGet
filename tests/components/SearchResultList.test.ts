import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import SearchResultList from '../../src/components/SearchResultList.vue';
import type { SearchResult } from '../../src/types';

describe('SearchResultList.vue', () => {
  const sampleResults: SearchResult[] = [
    {
      title: 'Doctor X：白色黑手黨時代',
      year: '2026',
      category: '韓劇',
      url: 'https://chinaq.fun/tv-kr/202639642/',
      source_id: 'chinaq',
      source_name: 'ChinaQ',
      cover_url: 'https://chinaq.fun/img/202639642.jpg',
    },
    {
      title: '繁星點點',
      year: null,
      category: '台劇',
      url: 'https://chinaq.fun/tv-tw/202209121/',
      source_id: 'chinaq',
      source_name: 'ChinaQ',
      cover_url: null,
    },
  ];

  it('renders cover image with correct src and loading lazy when cover_url exists', () => {
    const wrapper = mount(SearchResultList, {
      props: {
        results: sampleResults,
        searching: false,
        searched: true,
        keyword: 'Doctor',
      },
    });

    const coverImgs = wrapper.findAll('[data-testid="search-result-cover"]');
    expect(coverImgs.length).toBe(1);
    expect(coverImgs[0].attributes('src')).toBe('https://chinaq.fun/img/202639642.jpg');
    expect(coverImgs[0].attributes('loading')).toBe('lazy');
  });

  it('renders placeholder icon when cover_url is null', () => {
    const wrapper = mount(SearchResultList, {
      props: {
        results: sampleResults,
        searching: false,
        searched: true,
        keyword: '繁星',
      },
    });

    const placeholders = wrapper.findAll('[data-testid="search-result-placeholder"]');
    expect(placeholders.length).toBe(1);
  });

  it('falls back to placeholder when image fails to load', async () => {
    const wrapper = mount(SearchResultList, {
      props: {
        results: sampleResults,
        searching: false,
        searched: true,
        keyword: 'Doctor',
      },
    });

    const coverImg = wrapper.find('[data-testid="search-result-cover"]');
    expect(coverImg.exists()).toBe(true);

    // Trigger image loading error
    await coverImg.trigger('error');

    // Image should now be replaced by placeholder
    expect(wrapper.find('[data-testid="search-result-cover"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="search-result-placeholder"]').exists()).toBe(true);
  });

  it('emits select event when a result item is clicked', async () => {
    const wrapper = mount(SearchResultList, {
      props: {
        results: sampleResults,
        searching: false,
        searched: true,
        keyword: 'Doctor',
      },
    });

    const items = wrapper.findAll('[data-testid="search-result-item"]');
    await items[0].trigger('click');

    expect(wrapper.emitted('select')).toBeTruthy();
    expect(wrapper.emitted('select')![0]).toEqual([sampleResults[0]]);
  });

  it('renders source badges correctly for multiple sources', () => {
    const mixedResults: SearchResult[] = [
      {
        title: '慶餘年第一季',
        year: '2019',
        category: '陸劇',
        url: 'https://chinaq.fun/tv-cn/202614235/',
        source_id: 'chinaq',
        source_name: 'ChinaQ',
        cover_url: null,
      },
      {
        title: '慶餘年第二季',
        year: '2024',
        category: '陸劇',
        url: 'https://tw.juqing.app/vod/detail/id/gb27z2th.html',
        source_id: 'juqing',
        source_name: '小鴨影視',
        cover_url: 'https://image.juqing.app/vod/1c/01c92e9c3dd9bec34e0c7dfa022e2f24.webp',
      },
    ];

    const wrapper = mount(SearchResultList, {
      props: {
        results: mixedResults,
        searching: false,
        searched: true,
        keyword: '慶餘年',
      },
    });

    const sourceBadges = wrapper.findAll('[data-testid="search-result-source"]');
    expect(sourceBadges.length).toBe(2);
    expect(sourceBadges[0].text()).toBe(mixedResults[0].source_name);
    expect(sourceBadges[1].text()).toBe(mixedResults[1].source_name);
  });

  it('emits select event when juqing search result item is clicked', async () => {
    const juqingResult: SearchResult = {
      title: '慶餘年第二季',
      year: '2024',
      category: '陸劇',
      url: 'https://tw.juqing.app/vod/detail/id/gb27z2th.html',
      source_id: 'juqing',
      source_name: '小鴨影視',
      cover_url: 'https://image.juqing.app/vod/1c/01c92e9c3dd9bec34e0c7dfa022e2f24.webp',
    };

    const wrapper = mount(SearchResultList, {
      props: {
        results: [juqingResult],
        searching: false,
        searched: true,
        keyword: '慶餘年',
      },
    });

    const items = wrapper.findAll('[data-testid="search-result-item"]');
    await items[0].trigger('click');

    expect(wrapper.emitted('select')).toBeTruthy();
    expect(wrapper.emitted('select')![0]).toEqual([juqingResult]);
  });
});
