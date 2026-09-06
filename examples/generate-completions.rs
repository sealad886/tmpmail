use clap::CommandFactory;
use clap_complete::{Shell, generate_to};
use std::path::PathBuf;
use tmpmail::cli::Cli;

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: generate-completions OUTPUT");
    std::fs::create_dir_all(&output).expect("create output");
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        generate_to(shell, &mut Cli::command(), "tmpmail", &output).expect("generate completion");
    }
}
