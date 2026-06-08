/**
 * Settings-related type definitions for Aurora Launcher.
 * Matches the Rust backend's AppSettings and DownloadMirror structs.
 */

/** Download mirror options matching Rust DownloadMirror enum. */
export type DownloadMirror = 'Official' | 'Bmclapi';

/** Application settings matching the Rust backend's AppSettings. */
export interface AppSettings {
  download_mirror: DownloadMirror;
  default_max_memory: number;
  default_min_memory: number;
  download_concurrency: number;
  custom_java_path: string | null;
  window_width: number;
  window_height: number;
  language: string;
}

/** Default application settings. */
export const DEFAULT_SETTINGS: AppSettings = {
  download_mirror: 'Official',
  default_max_memory: 2048,
  default_min_memory: 512,
  download_concurrency: 8,
  custom_java_path: null,
  window_width: 1280,
  window_height: 800,
  language: 'zh-CN',
};
