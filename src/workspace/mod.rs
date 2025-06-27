// WezTerm Multi-Process Development Framework - Workspace Management Module

pub mod manager;
pub mod state;
pub mod template;
pub mod integration;

pub use manager::WorkspaceManager;
pub use state::{WorkspaceState, WorkspaceConfig};
pub use template::{WorkspaceTemplate, TemplateEngine};
pub use integration::IntegratedWorkspaceManager;