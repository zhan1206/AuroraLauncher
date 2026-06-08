/**
 * Download-related type definitions for Aurora Launcher.
 * Matches the Rust backend's download models.
 */

/** Download task status matching Rust DownloadStatus enum. */
export type DownloadStatus = 'Pending' | 'Downloading' | 'Paused' | 'Completed' | 'Failed' | 'Cancelled';

/** Download task record matching the Rust backend's DownloadTask struct. */
export interface DownloadTask {
  id: string;
  name: string;
  url: string;
  target_path: string;
  total_size: number;
  downloaded: number;
  status: DownloadStatus;
  sha256: string | null;
  concurrency: number;
  created_at: string;
}

/** Download progress event payload matching Rust DownloadProgress struct. */
export interface DownloadProgress {
  task_id: string;
  total: number;
  downloaded: number;
  speed: number;
  percent: number;
}
