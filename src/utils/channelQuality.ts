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

export function parseResolutionProfile(resStr: string): {
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
 * Returns the quality badge details for a dynamically probed stream, or null if unprobed.
 */
export function getStreamQualityBadge(
  customDetails?: StreamQualityInfo
): ChannelQualityBadge | null {
  if (!customDetails) {
    return null;
  }
  const profile = parseResolutionProfile(customDetails.resolution);
  return {
    resolution: profile.label,
    bitrate: customDetails.bitrate,
    tier: profile.tier,
  };
}
