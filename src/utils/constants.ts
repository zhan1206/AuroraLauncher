/**
 * Application-wide constants for Aurora Launcher.
 */

/** Application display name */
export const APP_NAME = "Aurora Launcher";

/** Current version */
export const APP_VERSION = "0.1.0";

/** Minecraft: Java Edition default directory names */
export const MC_DIR_NAME = ".minecraft";

/** Official Mojang meta URLs */
export const MOJANG_META_URL = "https://piston-meta.mojang.com";
export const VERSION_MANIFEST_URL =
  "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/** Microsoft OAuth endpoints */
export const MS_AUTH_URL = "https://login.microsoftonline.com";
export const MS_XBL_URL = "https://user.auth.xboxlive.com";
export const MS_XSTS_URL = "https://xsts.auth.xboxlive.com";
export const MS_MINECRAFT_URL = "https://api.minecraftservices.com";

/** Azure AD client ID for device code flow */
export const AZURE_CLIENT_ID = "00000000-0000-0000-0000-000000000000";

/** Error code ranges (matching Rust backend) */
export const ERROR_CODE = {
  NETWORK_START: 10000,
  FILE_START: 20000,
  AUTH_START: 30000,
  LAUNCH_START: 40000,
  CONFIG_START: 50000,
} as const;

/** Maximum concurrent downloads */
export const MAX_CONCURRENT_DOWNLOADS = 4;

/** Default JVM heap size in MB */
export const DEFAULT_MIN_MEMORY = 512;
export const DEFAULT_MAX_MEMORY = 2048;

/** Supported Minecraft version types */
export const VERSION_TYPES = {
  RELEASE: "release",
  SNAPSHOT: "snapshot",
  OLD_BETA: "old_beta",
  OLD_ALPHA: "old_alpha",
} as const;

/** Route paths */
export const ROUTES = {
  HOME: "/",
  INSTANCES: "/instances",
  INSTANCE_DETAIL: "/instances/:id",
  SETTINGS: "/settings",
} as const;
