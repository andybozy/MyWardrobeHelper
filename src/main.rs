mod api;
mod app;
mod cli;
mod config;
mod db;
mod domain;
mod error;
mod infra;
mod mcp;
mod repositories;
mod services;
mod web;

fn main() -> std::process::ExitCode {
    cli::run()
}
