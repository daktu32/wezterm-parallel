// WezTerm Multi-Process Development Framework - Workspace Management Module

pub mod integration;
pub mod manager;
pub mod state;
pub mod template;

pub use integration::IntegratedWorkspaceManager;
pub use manager::WorkspaceManager;
pub use state::{WorkspaceConfig, WorkspaceState};
pub use template::{TemplateEngine, WorkspaceTemplate};
