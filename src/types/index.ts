// Re-export generated schemas from Rust single source of truth
export type { EpisodeInfo } from './generated/EpisodeInfo';
export type { ChannelOption } from './generated/ChannelOption';
export type { SeriesMetadata } from './generated/SeriesMetadata';
export type { SearchResult } from './generated/SearchResult';
export type { SystemEnvironment } from './generated/SystemEnvironment';
export type { DownloadQueuePayload } from './generated/DownloadQueuePayload';
export type { ProgressPayload } from './generated/ProgressPayload';
export type { EpisodeStatusPayload } from './generated/EpisodeStatusPayload';
export type { DownloadTaskSpec } from './generated/DownloadTaskSpec';
export type { DownloadProgress } from './generated/DownloadProgress';

// UI Specific state models
export interface SeriesConfigModel {
  customTitle: string;
  season: number;
  saveDir: string;
  threads: number;
  maxConcurrent: number;
  binaryPath?: string;
}

export type QueueTaskStatus = 
  | { type: 'Queued' }
  | { type: 'Downloading' }
  | { type: 'Progress'; data: import('./generated/DownloadProgress').DownloadProgress }
  | { type: 'Muxing' }
  | { type: 'Completed' }
  | { type: 'Failed'; error: string }
  | { type: 'Cancelled' };

export interface QueueItem {
  id: number;
  spec: import('./generated/DownloadTaskSpec').DownloadTaskSpec;
  state: QueueTaskStatus;
  progress?: import('./generated/DownloadProgress').DownloadProgress;
}
