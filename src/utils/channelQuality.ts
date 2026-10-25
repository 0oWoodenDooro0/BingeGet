import type { ChannelOption } from '../types';

export interface ChannelQualityBadge {
  resolution: string;
  bitrate?: string;
  tier: 'optimal' | 'good' | 'medium';
}

export interface StreamQualityInfo {
  resolution: string;
  bitrate: string;
  video_codec?: string;
  audio_codec?: string;
}

// Known default baseline quality profiles for common streaming aggregators
const KNOWN_CHANNEL_BASELINES: Record<string, { rank: number; badge: ChannelQualityBadge }> = {
  ikzy: {
    rank: 3200,
    badge: { resolution: '1080p', bitrate: '3,200 kbps', tier: 'optimal' },
  },
  mdzy: {
    rank: 3120,
    badge: { resolution: '1080p', bitrate: '3,120 kbps', tier: 'optimal' },
  },
  hnzy: {
    rank: 2850,
    badge: { resolution: '1080p', bitrate: '2,850 kbps', tier: 'good' },
  },
  jyzy: {
    rank: 2600,
    badge: { resolution: '1080p', bitrate: '2,600 kbps', tier: 'good' },
  },
  lzzy: {
    rank: 2500,
    badge: { resolution: '1080p', bitrate: '2,500 kbps', tier: 'good' },
  },
  ffzy: {
    rank: 2400,
    badge: { resolution: '1080p', bitrate: '2,400 kbps', tier: 'good' },
  },
  jszy: {
    rank: 1750,
    badge: { resolution: '720p', bitrate: '1,750 kbps', tier: 'medium' },
  },
};

function parseResolutionProfile(resStr: string): {
  tier: 'optimal' | 'good' | 'medium';
  label: string;
  score: number;
} {
  const lower = resStr.toLowerCase();
  if (lower.includes('2160') || lower.includes('4k')) {
    return { tier: 'optimal', label: '4K', score: 10000 };
  }
  if (lower.includes('1080')) {
    return { tier: 'optimal', label: '1080p', score: 5000 };
  }
  if (lower.includes('720')) {
    return { tier: 'good', label: '720p', score: 2000 };
  }
  return { tier: 'medium', label: '標準畫質', score: 500 };
}

/**
 * Calculates a numerical quality rank score for a channel option.
 * Higher rank represents better resolution and bitrate.
 */
export function getChannelQualityRank(
  channelId: string,
  customDetails?: StreamQualityInfo
): number {
  if (customDetails) {
    const profile = parseResolutionProfile(customDetails.resolution);
    let score = profile.score;

    const brMatch = customDetails.bitrate.replace(/,/g, '').match(/(\d+)/);
    if (brMatch) {
      score += parseInt(brMatch[1], 10);
    }
    return score;
  }

  const baseline = KNOWN_CHANNEL_BASELINES[channelId];
  if (baseline) {
    return baseline.rank;
  }

  // Fallback rank for unknown channels
  return 100;
}

/**
 * Returns the quality badge details for a channel option.
 */
export function getChannelQualityBadge(
  channelId: string,
  customDetails?: StreamQualityInfo
): ChannelQualityBadge {
  if (customDetails) {
    const profile = parseResolutionProfile(customDetails.resolution);
    return {
      resolution: profile.label,
      bitrate: customDetails.bitrate,
      tier: profile.tier,
    };
  }

  const baseline = KNOWN_CHANNEL_BASELINES[channelId];
  if (baseline) {
    return baseline.badge;
  }

  return {
    resolution: '標準畫質',
    tier: 'medium',
  };
}

/**
 * Sorts channel options descending by stream quality (highest quality first).
 */
export function sortChannelsByQuality(
  channels: ChannelOption[],
  probeMap?: Record<string, StreamQualityInfo>
): ChannelOption[] {
  return [...channels].sort((a, b) => {
    const rankA = getChannelQualityRank(a.id, probeMap?.[a.id]);
    const rankB = getChannelQualityRank(b.id, probeMap?.[b.id]);
    return rankB - rankA;
  });
}
