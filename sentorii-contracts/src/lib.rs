#![forbid(unsafe_code)]
#![deny(warnings)]

//! # Sentorii Contracts
//!
//! This crate is the single source of truth for all shared data types and traits
//! used for communication between the core engine and the user interfaces of the
//! Sentorii application.
//!
//! It is a foundational, data-centric library with zero business logic. its primary
//! purpose is to define the "language" that different components of the application
//! use to talk to each other.

pub mod command;
pub mod event;
pub mod runner;
pub mod step;
pub mod ui;
pub mod workflow_request;
