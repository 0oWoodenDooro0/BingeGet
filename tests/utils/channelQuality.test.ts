import { describe, it, expect } from 'vitest';
import {
  getChannelQualityRank,
  getChannelQualityBadge,
  sortChannelsByQuality,
} from '../../src/utils/channelQuality';
import type { ChannelOption } from '../../src/types';

describe('channelQuality utils', () => {
  const sampleChannels: ChannelOption[] = [
    { id: 'jszy', name: '極速 資源 (jszy)' },
    { id: 'hnzy', name: '紅牛 資源 (hnzy)' },
    { id: 'ikzy', name: 'iKun 資源 (ikzy)' },
    { id: 'unknown_source', name: '未知 片源 (unknown)' },
  ];

  it('calculates higher quality rank for 1080p high bitrate channels than 720p', () => {
    const rankIkzy = getChannelQualityRank('ikzy');
    const rankHnzy = getChannelQualityRank('hnzy');
    const rankJszy = getChannelQualityRank('jszy');

    expect(rankIkzy).toBeGreaterThan(rankHnzy);
    expect(rankHnzy).toBeGreaterThan(rankJszy);
  });

  it('sorts channels descending from highest quality to lowest quality', () => {
    const sorted = sortChannelsByQuality(sampleChannels);

    expect(sorted.map(c => c.id)).toEqual(['ikzy', 'hnzy', 'jszy', 'unknown_source']);
  });

  it('uses dynamic probed details to prioritize newly probed higher bitrate channels', () => {
    const customProbeMap = {
      hnzy: {
        resolution: '1920x1080',
        bitrate: '5,000 kbps',
        video_codec: 'H.264',
        audio_codec: 'AAC',
      },
      ikzy: {
        resolution: '1920x1080',
        bitrate: '3,200 kbps',
        video_codec: 'H.264',
        audio_codec: 'AAC',
      },
    };

    const sorted = sortChannelsByQuality(sampleChannels, customProbeMap);
    // With 5000 kbps, hnzy should outrank ikzy
    expect(sorted[0].id).toBe('hnzy');
    expect(sorted[1].id).toBe('ikzy');
  });

  it('returns quality badge info including resolution label and tier', () => {
    const badgeIkzy = getChannelQualityBadge('ikzy');
    expect(badgeIkzy.resolution).toBe('1080p');
    expect(badgeIkzy.tier).toBe('optimal');

    const badgeJszy = getChannelQualityBadge('jszy');
    expect(badgeJszy.resolution).toBe('720p');
    expect(badgeJszy.tier).toBe('medium');
  });
});
