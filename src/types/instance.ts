/**
 * Instance-related type definitions for Aurora Launcher.
 * Matches the Rust backend's Instance and related structs.
 */

/** Supported Minecraft loader types matching Rust LoaderType enum. */
export type LoaderType = 'Vanilla' | 'Forge' | 'Fabric' | 'NeoForge' | 'Quilt';

/** Instance record matching the Rust backend's Instance struct. */
export interface Instance {
  id: string;
  name: string;
  version_id: string;
  loader_type: LoaderType;
  loader_version: string | null;
  game_dir: string;
  java_id: string | null;
  launch_config: string;
  created_at: string;
  updated_at: string;
  icon: string | null;
  notes: string | null;
}

/** Launch configuration embedded in an instance matching Rust LaunchConfig. */
export interface LaunchConfig {
  min_memory: number;
  max_memory: number;
  jvm_args: string[];
  game_args: string[];
  fullscreen: boolean;
  width: number;
  height: number;
}

/** Request body for creating a new instance matching Rust CreateInstanceRequest. */
export interface CreateInstanceRequest {
  name: string;
  version_id: string;
  loader_type?: string;
  loader_version?: string;
  launch_config?: LaunchConfig;
  icon?: string;
  notes?: string;
}

/** Request body for updating an existing instance matching Rust UpdateInstanceRequest. */
export interface UpdateInstanceRequest {
  id: string;
  name?: string;
  version_id?: string;
  loader_type?: string;
  loader_version?: string;
  java_id?: string | null;
  launch_config?: LaunchConfig;
  icon?: string;
  notes?: string;
}

/** Instance status for UI display (frontend-only). */
export type InstanceStatus = 'idle' | 'running' | 'launching' | 'updating';
