//! Domain layer containing business logic and data models
//!
//! This module contains the core domain models and business logic
//! for the ToDo API and Criminal Case Management applications.

pub mod attorney;
pub mod criminal_case;
pub mod deadline;
pub mod docket;
pub mod features;
pub mod judge;
pub mod opinion;
pub mod order;
pub mod sentencing;
mod todo;

pub use todo::ToDo;
