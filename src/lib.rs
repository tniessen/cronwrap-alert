use std::path::Path;
use std::process::{Command, ExitStatus, Output};

use chrono::format::SecondsFormat;
use chrono::prelude::Utc;
use clap::Parser;
use gethostname::gethostname;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct SuccessResponse {
    key: String,
}

struct AlertConfig {
    pub endpoint: String,
    pub channel: String,
    pub category: Option<String>,
    pub origin: Option<String>,
    pub token: String,
}

fn post_alert(
    config: &AlertConfig,
    subject: &str,
    content_type: &str,
    body: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let alert = json!({
        "channel": config.channel,
        "category": config.category,
        "origin": config.origin,
        "subject": subject,
        "contentType": content_type,
        "body": body
    });

    let response = reqwest::blocking::Client::new()
        .post(&config.endpoint)
        .header("Authorization", format!("Bearer {}", config.token))
        .json(&alert)
        .send()?
        .json::<SuccessResponse>()?;

    Ok(response.key)
}

fn send_alert(config: &AlertConfig, subject: &str, body: &str) {
    eprintln!("Command failed. Sending alert...");
    match post_alert(config, subject, "text/plain", body) {
        Ok(key) => eprintln!("Alert sent. Key: {}", key),
        Err(e) => eprintln!("Failed to send alert: {e}"),
    }
}

#[derive(Parser)]
#[command(
    author,
    version,
    max_term_width = 95,
    after_help = "All --alert-* options take precedence over environment variables \
     CRONWRAP_ALERT_*. In turn, CRONWRAP_ALERT_* environment variables take \
     precedence over ALERT_* environment variables."
)]
pub struct Args {
    /// Alert channel (defaults to "cronwrap")
    #[arg(long, value_name = "CHANNEL")]
    pub alert_channel: Option<String>,

    /// Alert category (optional)
    #[arg(long, value_name = "CATEGORY")]
    pub alert_category: Option<String>,

    /// Alert origin (defaults to the hostname)
    #[arg(long, value_name = "ORIGIN", group = "alert_origin_group")]
    pub alert_origin: Option<String>,

    /// Disable the default alert origin (hostname)
    #[arg(long, group = "alert_origin_group")]
    pub no_alert_origin: bool,

    /// Alert endpoint URI (required)
    #[arg(long, value_name = "URI")]
    pub alert_endpoint: Option<String>,

    /// Authorization token for alert endpoint (required)
    #[arg(long, value_name = "TOKEN")]
    pub alert_token: Option<String>,

    /// Maximum size of the standard error output to include in the alert
    #[arg(long, default_value_t = 65536, value_name = "N")]
    pub max_output_capture: usize,

    /// Non-zero exit codes to treat as successful
    #[arg(long, value_name = "EXIT_CODE")]
    pub ignore_exit_code: Vec<i32>,

    /// Program name for use in alert messages
    #[arg(long, value_name = "NAME")]
    pub program_name: Option<String>,

    /// Command to run
    pub command: String,

    /// Additional arguments
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

fn format_spawn_error(
    start_time: &str,
    program_name: &str,
    args: &Args,
    e: std::io::Error,
) -> String {
    let json_command = json!(args.command).to_string();
    let mut out = format!(
        "Failed to start {} at {}: {}\n\nCommand: {}\n\nArguments:\n",
        program_name, start_time, e, json_command
    );
    for arg in &args.args {
        out.push_str(&format!("\n  {}", json!(arg).to_string()));
    }
    out
}

fn format_execution_error(
    start_time: &str,
    program_name: &str,
    args: &Args,
    output: &Output,
) -> String {
    let desc = if let Some(code) = output.status.code() {
        format!("exited with status code {}", code)
    } else {
        "was terminated by a signal".to_string()
    };
    let json_command = json!(args.command).to_string();
    let mut out = format!(
        "Execution of {} at {} failed: {}\n\nCommand: {}\n\nArguments:\n",
        program_name, start_time, desc, json_command
    );
    for arg in &args.args {
        out.push_str(&format!("\n  {}", json!(arg).to_string()));
    }
    let stderr_string = String::from_utf8_lossy(&output.stderr);
    if stderr_string.is_empty() {
        out.push_str("\n\nStandard error output (empty):\n");
    } else {
        if stderr_string.len() > args.max_output_capture {
            out.push_str("\n\nStandard error output (truncated):\n\n[...]\n");
            let partial = {
                let pos = stderr_string
                    .char_indices()
                    .nth_back(args.max_output_capture)
                    .unwrap()
                    .0;
                &stderr_string[pos..]
            };
            out.push_str(partial);
        } else {
            out.push_str("\n\nStandard error output:\n\n");
            out.push_str(&stderr_string);
        }
    }
    out
}

fn get_program_name(args: &Args) -> String {
    let path = Path::new(&args.command);
    let default_name: String = path
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| args.command.clone());
    args.program_name
        .as_ref()
        .unwrap_or(&default_name)
        .to_string()
}

fn is_success(status: &ExitStatus, ignored: &[i32]) -> bool {
    status.success() || status.code().map_or(false, |c| ignored.contains(&c))
}

fn or_env(value: &Option<String>, var_name: &str) -> Option<String> {
    value
        .clone()
        .or_else(|| std::env::var(format!("CRONWRAP_{}", var_name)).ok())
        .or_else(|| std::env::var(var_name).ok())
}

pub fn main(args: Args) -> u8 {
    let maybe_hostname: Option<String> = if args.no_alert_origin {
        None
    } else {
        Some(gethostname().to_string_lossy().into_owned())
    };
    let config = AlertConfig {
        endpoint: or_env(&args.alert_endpoint, "ALERT_ENDPOINT").expect(
            "One of --alert-endpoint, CRONWRAP_ALERT_ENDPOINT, or ALERT_ENDPOINT must be set",
        ),
        channel: or_env(&args.alert_channel, "ALERT_CHANNEL").unwrap_or("cronwrap".to_string()),
        category: or_env(&args.alert_category, "ALERT_CATEGORY"),
        origin: or_env(&args.alert_origin, "ALERT_ORIGIN").or(maybe_hostname),
        token: or_env(&args.alert_token, "ALERT_TOKEN")
            .expect("One of --alert-token, CRONWRAP_ALERT_TOKEN, or ALERT_TOKEN must be set"),
    };

    let program_name = get_program_name(&args);
    let body: String;
    let start_time = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    match Command::new(&args.command).args(&args.args).output() {
        Ok(output) => {
            if is_success(&output.status, &args.ignore_exit_code) {
                return 0;
            }
            body = format_execution_error(&start_time, program_name.as_str(), &args, &output);
        }
        Err(e) => {
            body = format_spawn_error(&start_time, program_name.as_str(), &args, e);
        }
    }

    let subject = format!("Scheduled execution failed: {}", program_name);
    send_alert(&config, subject.as_str(), body.as_str());

    1
}
