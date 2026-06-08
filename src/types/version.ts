/**
 * Version-related type definitions for Aurora Launcher.
 * Matches the Rust backend's version structs.
 */

/** Version entry in the manifest listing. */
export interface VersionEntry {
  id: string;
  type: string;
  url: string;
  release_time: string;
}

/** Version manifest from Mojang's API. */
export interface VersionManifest {
  latest: {
    release: string;
    snapshot: string;
  };
  versions: VersionEntry[];
}

/** Library download artifact. */
export interface LibraryArtifact {
  path: string;
  sha1: string;
  size: number;
  url: string;
}

/** A game library dependency. */
export interface Library {
  name: string;
  downloads: {
    artifact?: LibraryArtifact;
    classifiers?: Record<string, LibraryArtifact>;
  };
  rules?: Array<{
    action: 'allow' | 'disallow';
    os?: {
      name: string;
    };
  }>;
  natives?: Record<string, string>;
}

/** Download info for a version's client jar. */
export interface VersionDownload {
  sha1: string;
  size: number;
  url: string;
}

/** Version detail after fetching version JSON. */
export interface VersionDetail {
  id: string;
  type: string;
  main_class: string;
  libraries: Library[];
  downloads: {
    client?: VersionDownload;
    server?: VersionDownload;
    client_mappings?: VersionDownload;
    server_mappings?: VersionDownload;
  };
  asset_index: {
    id: string;
    sha1: string;
    size: number;
    total_size: number;
    url: string;
  };
  release_time: string;
  java_version?: {
    component: string;
    major_version: number;
  };
}

/** Version type filter options. */
export type VersionTypeFilter = 'all' | 'release' | 'snapshot' | 'old_beta' | 'old_alpha';
