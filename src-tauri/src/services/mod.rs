//! Service layer modules.
//!
//! Each service encapsulates business logic for a specific domain:
//! version management, instance CRUD, download orchestration, Java runtime
//! management, mod loader installation, and settings persistence.

pub mod version_service;
pub mod instance_service;
pub mod download_service;
pub mod java_service;
pub mod loader_service;
pub mod settings_service;
pub mod account_service;
pub mod launch_service;
