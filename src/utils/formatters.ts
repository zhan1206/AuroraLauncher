/**
 * Formatting utilities for Aurora Launcher.
 * Provides consistent formatting for file sizes, dates, durations, etc.
 */

/** Format bytes into a human-readable file size string. */
export function formatFileSize(bytes: number): string {
  if (bytes < 0) return '0 B';
  if (bytes === 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const index = Math.min(i, units.length - 1);
  const value = bytes / Math.pow(k, index);
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

/** Format a date string (ISO 8601) into a locale-friendly display. */
export function formatDate(dateStr: string | null, locale: string = 'zh-CN'): string {
  if (!dateStr) return '从未';
  const date = new Date(dateStr);
  if (isNaN(date.getTime())) return '无效日期';
  return date.toLocaleDateString(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  });
}

/** Format a date string into a full datetime display. */
export function formatDateTime(dateStr: string | null, locale: string = 'zh-CN'): string {
  if (!dateStr) return '从未';
  const date = new Date(dateStr);
  if (isNaN(date.getTime())) return '无效日期';
  return date.toLocaleString(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/** Format a relative time string (e.g. "3 分钟前"). */
export function formatRelativeTime(dateStr: string | null): string {
  if (!dateStr) return '从未';
  const date = new Date(dateStr);
  if (isNaN(date.getTime())) return '无效日期';

  const now = Date.now();
  const diff = now - date.getTime();
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (seconds < 60) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  if (hours < 24) return `${hours} 小时前`;
  if (days < 30) return `${days} 天前`;
  return formatDate(dateStr);
}

/** Format play time from seconds into a human-readable duration. */
export function formatDuration(seconds: number): string {
  if (seconds <= 0) return '0 分钟';
  if (seconds < 60) return `${Math.floor(seconds)} 秒`;
  if (seconds < 3600) {
    const mins = Math.floor(seconds / 60);
    return `${mins} 分钟`;
  }
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  if (mins > 0) {
    return `${hours} 小时 ${mins} 分钟`;
  }
  return `${hours} 小时`;
}

/** Format a download speed in bytes per second to a readable string. */
export function formatSpeed(bytesPerSec: number): string {
  return `${formatFileSize(bytesPerSec)}/s`;
}

/** Format an ETA in seconds to a readable string. */
export function formatEta(seconds: number): string {
  if (seconds <= 0 || !isFinite(seconds)) return '--';
  if (seconds < 60) return `${Math.ceil(seconds)} 秒`;
  if (seconds < 3600) {
    const mins = Math.floor(seconds / 60);
    const secs = Math.ceil(seconds % 60);
    return `${mins} 分 ${secs} 秒`;
  }
  const hours = Math.floor(seconds / 3600);
  const mins = Math.ceil((seconds % 3600) / 60);
  return `${hours} 时 ${mins} 分`;
}

/** Format a percentage (0-100) with one decimal. */
export function formatPercent(value: number): string {
  if (value <= 0) return '0%';
  if (value >= 100) return '100%';
  return `${value.toFixed(1)}%`;
}

/** Format memory size in MB to a display string. */
export function formatMemory(mb: number): string {
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(1)} GB`;
  }
  return `${mb} MB`;
}

/** Capitalize the first letter of a string. */
export function capitalize(str: string): string {
  if (!str) return '';
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/** Format a loader type to a display-friendly name. */
export function formatLoaderType(loader: string): string {
  const map: Record<string, string> = {
    vanilla: 'Vanilla',
    forge: 'Forge',
    fabric: 'Fabric',
    neoforge: 'NeoForge',
    quilt: 'Quilt',
  };
  return map[loader] ?? capitalize(loader);
}

/** Format a version type to a display-friendly name. */
export function formatVersionType(type: string): string {
  const map: Record<string, string> = {
    release: '正式版',
    snapshot: '快照',
    old_beta: '旧版 Beta',
    old_alpha: '旧版 Alpha',
  };
  return map[type] ?? capitalize(type);
}
