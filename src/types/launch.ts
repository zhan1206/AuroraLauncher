/**
 * Launch-related type definitions for Aurora Launcher.
 */

/** Launch status for tracking game startup. */
export type LaunchStatus = 'idle' | 'preparing' | 'installing' | 'launching' | 'running' | 'exited' | 'crashed';

/** Game log entry from the running Minecraft process. */
export interface GameLogEntry {
  timestamp: string;
  level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG' | 'TRACE' | 'FATAL';
  source: string;
  message: string;
}

/** Launch started event payload. */
export interface LaunchStartedEvent {
  instance_id: string;
  pid: number;
}

/** Launch exited event payload. */
export interface LaunchExitedEvent {
  instance_id: string;
  exit_code: number;
  duration_seconds: number;
}

/** Game log event payload. */
export interface GameLogEvent {
  instance_id: string;
  entry: GameLogEntry;
}
