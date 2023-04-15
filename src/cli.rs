use anyhow::anyhow;
use clap::Parser;

/// Fetch weather information periodically
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to file containing Telegram Bot Token
    #[arg(long, env)]
    pub api_token_file: Option<String>,

    /// Bot token. **Highly recommended that this is not set via command line, because it will show up in running processes.**
    #[arg(long, env, required_unless_present("api_token_file"))]
    pub api_token: Option<String>,
}

impl Cli {
    pub fn get_token(&self) -> anyhow::Result<String> {
        get_token(self.api_token.as_ref(), self.api_token_file.as_ref())
    }
}

fn get_token<S1, S2>(token: Option<S1>, file: Option<S2>) -> anyhow::Result<String>
where
    S1: std::string::ToString,
    S2: AsRef<std::path::Path>,
{
    if let Some(token) = token.as_ref() {
        return Ok(token.to_string());
    }
    if let Some(file) = file.as_ref() {
        return Ok(std::fs::read_to_string(file)?.trim().to_string());
    }
    Err(anyhow!("No API Key provided"))
}
