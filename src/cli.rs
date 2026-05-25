const BOOTSTRAP_MESSAGE: &str = "\
MyWardrobeHelper bootstrap is ready.

Current state:
- repository guidance and backlog tracking are initialized
- Rust backend module layout is scaffolded
- CLI subcommands, HTTP server, API, MCP, and iOS workflows are still ahead in TODO.md

Next planned backend commands:
- serve
- init
- doctor
- backup
- export
- mcp serve
";

pub fn run() {
    println!("{BOOTSTRAP_MESSAGE}");
}
