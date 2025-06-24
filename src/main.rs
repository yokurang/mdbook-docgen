use chrono::Local;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use convert_case::{Case, Casing};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scaffold a docs folder with mdBook & plugins configured
    Init {
        name: String,
        /// target directory (default: ./docs)
        #[arg(default_value = "docs")]
        dir: PathBuf,
    },
    /// Build the book (static site goes to <dir>/book)
    Build {
        #[arg(default_value = "docs")]
        dir: PathBuf,
    },
    /// live-reload server (mdbook serve --open)
    Serve {
        #[arg(default_value = "docs")]
        dir: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Init { name, dir } => init(&name, &dir),
        Cmd::Build { dir } => build(&dir, false),
        Cmd::Serve { dir } => build(&dir, true),
    }
}

fn init(name: &str, dir: &Path) -> Result<()> {
    if dir.exists() {
        bail!("{dir:?} already exists – aborting");
    }

    let status = Command::new("mdbook")
        .args(["init", "--title", name, dir.to_str().unwrap()])
        .status();

    match status {
        Ok(s) if s.success() => (),
        Ok(s) => bail!("mdbook failed with exit code {}", s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            bail!("mdbook not found. Please run `cargo install mdbook`.")
        }
        Err(e) => return Err(e.into()),
    }

    let book_toml = dir.join("book.toml");
    let mut cfg = fs::read_to_string(&book_toml)?;
    cfg.push_str(
        r#"
[preprocessor.plantuml]
command       = "mdbook-plantuml"
plantuml-cmd  = "plantuml"
use-data-uris = true

[preprocessor.fs-summary]
command = "mdbook-fs-summary"
write-summary = true

[preprocessor.mermaid]
command = "mdbook-mermaid"

[output]

[output.html]
additional-js = ["mermaid.min.js", "mermaid-init.js"]
"#,
    );
    fs::write(&book_toml, cfg)?;

    println!("Docs scaffold generated at {}", dir.display());
    println!(
        "Put Markdown anywhere under {}/src and run `cargo run -- build`",
        dir.display()
    );
    Ok(())
}

fn build(dir: &Path, serve: bool) -> Result<()> {
    let src = dir.join("src");
    if src.exists() {
        ensure_index_md(&src)?;  
        gen_summary(&src)?;      

        let book_title =
            find_book_title(&dir.join("book.toml")).unwrap_or_else(|| "Overview".to_string());
        ensure_overview(&src, &book_title)?;
    }


    let mut cmd = Command::new("mdbook");
    cmd.current_dir(dir);
    if serve {
        cmd.args(["serve", "--open"]);
    } else {
        cmd.arg("build");
    }

    let status = cmd.status().context("failed to run mdbook")?;
    if !status.success() {
        bail!("mdbook exited with {:?}", status);
    }

    if !serve {
        println!("Built site: {}/book/index.html", dir.display());
    }
    Ok(())
}

/// tiny SUMMARY.md generator (alphabetical).  Plugin will overwrite it later.
fn gen_summary(src: &Path) -> Result<()> {
    let mut lines = vec!["# Summary".into()];
    for entry in WalkDir::new(src)
        .sort_by(|a, b| a.path().cmp(b.path()))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
    {
        let rel = entry.path().strip_prefix(src)?;
        let depth = rel.components().count() - 1;
        let title = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .replace('_', " ")
            .to_case(Case::Title);
        let indent = " ".repeat(depth * 4);
        lines.push(format!("{indent}- [{title}]({})", rel.display()));
    }
    let mut f = File::create(src.join("SUMMARY.md"))?;
    for l in lines {
        writeln!(f, "{l}")?;
    }
    Ok(())
}

/// Ensure every subfolder in src/ has an index.md file.
fn ensure_index_md(src: &Path) -> Result<()> {
    let root_idx = src.join("00.md");
    if root_idx.exists() {
        fs::remove_file(&root_idx)?;
    }
    for entry in WalkDir::new(src)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
    {
        let dir = entry.path();

        if dir.strip_prefix(src)?
        .starts_with("mdbook-plantuml-img") {
            continue;
        }

        if dir == src {
            continue;
        }

        let idx = dir.join("00.md");
        if !idx.exists() {
            let title = dir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .replace('_', " ")
                .to_case(Case::Title);
            let mut f = File::create(&idx)?;
            writeln!(f, "# {title}\n")?;
            println!("Created {}", idx.display());
        }
    }
    Ok(())
}


/// Always overwrite `src/overview/00.md` with a fresh TOC.
fn ensure_overview(src: &Path, _book_title: &str) -> Result<()> {
    use std::io::Write;

    // make sure overview/ exists
    let ov_md = src.join("overview").join("00.md");
    fs::create_dir_all(ov_md.parent().unwrap())?;

    // read SUMMARY.md and filter
    let summary_text = fs::read_to_string(src.join("SUMMARY.md")).unwrap_or_default();
    let mut toc_lines = Vec::new();

    for line in summary_text.lines() {
        let trimmed = line.trim_start();

        // skip the "# Summary" header & blank lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        // skip root & folder landing pages
        if trimmed.contains("00.md)") || trimmed.contains("SUMMARY.md)") {
            continue;
        }

        // update relative path so links resolve from overview/
        // "(test1/foo.md)"  ->  "(../test1/foo.md)"
        // toc_lines.push(line.replacen("](", "](../", 1));
        toc_lines.push(line);
    }

    let mut f = File::create(&ov_md)?;
    writeln!(f, "# Overview\n")?;
    writeln!(
        f,
        "> _Auto-generated on {}_\n",
        Local::now().format("%Y-%m-%d")
    )?;
    writeln!(f, "## Table of contents\n")?;
    for l in toc_lines {
        writeln!(f, "{l}")?;
    }
    writeln!(
        f,
        "\n---\n\n_Edit this page to add project goals, context diagrams, etc._"
    )?;
    println!("📄  Updated {}", ov_md.display());
    Ok(())
}

fn find_book_title(toml: &Path) -> Option<String> {
    let contents = fs::read_to_string(toml).ok()?;
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("title") {
            return rest
                .split('=')
                .nth(1)
                .map(|s| s.trim().trim_matches('"').to_string());
        }
    }
    None
}
