export interface EpisodeInfo {
  ep_number: number;
  ep_id: string;
}

export interface ChannelOption {
  id: string;
  name: string;
}

export interface SeriesMetadata {
  drama_id: string;
  episodes: EpisodeInfo[];
  channels: ChannelOption[];
}

export interface DownloadTaskSpec {
  m3u8_url: string;
  custom_title: string;
  season: number;
  episode: number;
  save_dir: string;
  referer: string;
}

export interface DownloadProgress {
  percent: number;
  speed: string;
  eta: string;
}

export type QueueTaskStatus = 
  | { type: 'Queued' }
  | { type: 'Downloading' }
  | { type: 'Progress'; data: DownloadProgress }
  | { type: 'Muxing' }
  | { type: 'Completed' }
  | { type: 'Failed'; error: string }
  | { type: 'Cancelled' };

export interface QueueItem {
  id: number;
  spec: DownloadTaskSpec;
  state: QueueTaskStatus;
}
