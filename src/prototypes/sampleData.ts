import type { SeriesMetadata, SeriesConfigModel, StreamDetails } from '../types';

export const SAMPLE_METADATA: SeriesMetadata = {
  drama_id: 'sample-drama-42',
  category_slug: 'drama',
  cover_url: null,
  channels: [
    { id: 'ikzy', name: 'iKun 資源 (ikzy)' },
    { id: 'mdzy', name: '魔都 資源 (mdzy)' },
    { id: 'hnzy', name: '紅牛 資源 (hnzy)' },
    { id: 'jszy', name: '極速 資源 (jszy)' },
  ],
  episodes: Array.from({ length: 30 }, (_, i) => ({
    ep_number: i + 1,
    ep_id: `ep-${i + 1}`,
    available_channels: ['ikzy', 'mdzy', 'hnzy', 'jszy'],
  })),
};

export const SAMPLE_CONFIG: SeriesConfigModel = {
  customTitle: '慶餘年 第二季',
  season: 2,
  saveDir: '/home/user/Downloads',
  threads: 16,
  maxConcurrent: 2,
};

export const SAMPLE_STREAM_DETAILS: Record<string, StreamDetails> = {
  ikzy: {
    resolution: '1080p',
    bitrate: '3,200 kbps',
    video_codec: 'H.264',
    audio_codec: 'AAC',
    fps: 30,
    duration_secs: 2700,
  },
  mdzy: {
    resolution: '1080p',
    bitrate: '3,120 kbps',
    video_codec: 'H.264',
    audio_codec: 'AAC',
    fps: 30,
    duration_secs: 2700,
  },
  hnzy: {
    resolution: '1080p',
    bitrate: '2,850 kbps',
    video_codec: 'H.264',
    audio_codec: 'AAC',
    fps: 25,
    duration_secs: 2700,
  },
  jszy: {
    resolution: '720p',
    bitrate: '1,750 kbps',
    video_codec: 'H.264',
    audio_codec: 'AAC',
    fps: 25,
    duration_secs: 2700,
  },
};
