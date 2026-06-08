/**
 * Validation utilities for Aurora Launcher.
 * Provides input validation for usernames, instance names, etc.
 */

/** Validation result with optional error message. */
export interface ValidationResult {
  valid: boolean;
  message: string;
}

/** Create a successful validation result. */
function ok(): ValidationResult {
  return { valid: true, message: '' };
}

/** Create a failed validation result with an error message. */
function fail(message: string): ValidationResult {
  return { valid: false, message };
}

/** Validate a Minecraft username. */
export function validateUsername(username: string): ValidationResult {
  if (!username || username.trim().length === 0) {
    return fail('用户名不能为空');
  }
  const trimmed = username.trim();
  if (trimmed.length < 3) {
    return fail('用户名至少需要 3 个字符');
  }
  if (trimmed.length > 16) {
    return fail('用户名不能超过 16 个字符');
  }
  if (!/^[a-zA-Z0-9_]+$/.test(trimmed)) {
    return fail('用户名只能包含字母、数字和下划线');
  }
  return ok();
}

/** Validate an instance name. */
export function validateInstanceName(name: string): ValidationResult {
  if (!name || name.trim().length === 0) {
    return fail('实例名称不能为空');
  }
  const trimmed = name.trim();
  if (trimmed.length < 1) {
    return fail('实例名称不能为空');
  }
  if (trimmed.length > 64) {
    return fail('实例名称不能超过 64 个字符');
  }
  // Disallow characters that are invalid in file paths
  const invalidChars = /[<>:"/\\|?*\x00-\x1f]/;
  if (invalidChars.test(trimmed)) {
    return fail('实例名称包含非法字符');
  }
  // Disallow names that end with a dot or space (Windows issue)
  if (trimmed.endsWith('.') || trimmed.endsWith(' ')) {
    return fail('实例名称不能以点号或空格结尾');
  }
  return ok();
}

/** Validate a Java path. */
export function validateJavaPath(path: string): ValidationResult {
  if (!path || path.trim().length === 0) {
    return ok(); // Empty means auto-detect, which is valid
  }
  const trimmed = path.trim();
  if (trimmed.length > 512) {
    return fail('Java 路径过长');
  }
  // Basic check for java/javaw executable
  const lower = trimmed.toLowerCase();
  if (!lower.includes('java') && !lower.includes('javaw')) {
    return fail('路径似乎不是有效的 Java 可执行文件');
  }
  return ok();
}

/** Validate memory size in MB. */
export function validateMemory(mb: number, min: number = 128, max: number = 65536): ValidationResult {
  if (!Number.isFinite(mb)) {
    return fail('请输入有效的数字');
  }
  if (mb < min) {
    return fail(`内存不能小于 ${min} MB`);
  }
  if (mb > max) {
    return fail(`内存不能大于 ${max} MB`);
  }
  return ok();
}

/** Validate min/max memory pair. */
export function validateMemoryPair(minMb: number, maxMb: number): ValidationResult {
  const minResult = validateMemory(minMb);
  if (!minResult.valid) return minResult;
  const maxResult = validateMemory(maxMb);
  if (!maxResult.valid) return maxResult;
  if (minMb > maxMb) {
    return fail('最小内存不能大于最大内存');
  }
  return ok();
}

/** Validate JVM arguments string. */
export function validateJvmArgs(args: string): ValidationResult {
  if (!args || args.trim().length === 0) {
    return ok(); // Empty is valid (means default)
  }
  // Check for obviously dangerous arguments
  const dangerous = /-Xm[sx]\s*\d+[kmgKMG]/;
  if (dangerous.test(args)) {
    return fail('请不要在 JVM 参数中设置内存大小，请使用内存配置');
  }
  return ok();
}

/** Validate a download mirror value. */
export function validateDownloadMirror(mirror: string): ValidationResult {
  const validMirrors = ['official', 'bmclapi', 'mcbbs'];
  if (!validMirrors.includes(mirror)) {
    return fail('无效的下载源');
  }
  return ok();
}

/** Validate a version ID. */
export function validateVersionId(versionId: string): ValidationResult {
  if (!versionId || versionId.trim().length === 0) {
    return fail('请选择游戏版本');
  }
  return ok();
}
