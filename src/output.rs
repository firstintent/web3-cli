use serde::Serialize;
use tabled::settings::object::Columns;
use tabled::settings::{Modify, Style, Width};
use tabled::Table;

use crate::error::Web3CliError;

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Serialize)]
struct JsonEnvelope<T: Serialize> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<crate::error::ErrorResponse>,
}

pub fn print_json<T: Serialize>(data: &T) -> anyhow::Result<()> {
    let envelope = JsonEnvelope::<&T> {
        ok: true,
        chain: None,
        network: None,
        data: Some(data),
        error: None,
    };
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}

pub fn print_json_with_chain<T: Serialize>(
    data: &T,
    chain: &str,
    network: &str,
) -> anyhow::Result<()> {
    let envelope = JsonEnvelope::<&T> {
        ok: true,
        chain: Some(chain.to_string()),
        network: Some(network.to_string()),
        data: Some(data),
        error: None,
    };
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}

pub fn print_error(err: &Web3CliError, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let envelope = JsonEnvelope::<()> {
                ok: false,
                chain: None,
                network: None,
                data: None,
                error: Some(err.to_error_response()),
            };
            let json = serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| {
                format!(
                    r#"{{"ok":false,"error":{{"code":"INTERNAL_ERROR","message":"{}","category":"internal"}}}}"#,
                    err.to_string().replace('"', "'")
                )
            });
            println!("{json}");
        }
        OutputFormat::Table => {
            eprintln!("Error: {err}");
        }
    }
}

pub fn print_detail_table(rows: Vec<[String; 2]>) {
    let table = Table::from_iter(rows)
        .with(Style::rounded())
        .with(Modify::new(Columns::first()).with(Width::wrap(20)))
        .with(Modify::new(Columns::last()).with(Width::wrap(80)))
        .to_string();
    println!("{table}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_envelope_success() {
        let envelope = JsonEnvelope {
            ok: true,
            chain: Some("ethereum".into()),
            network: Some("mainnet".into()),
            data: Some(serde_json::json!({"balance": "1.5"})),
            error: None,
        };
        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["chain"], "ethereum");
        assert!(json["error"].is_null());
    }

    #[test]
    fn json_envelope_error_omits_data() {
        let envelope = JsonEnvelope::<()> {
            ok: false,
            chain: None,
            network: None,
            data: None,
            error: Some(crate::error::ErrorResponse {
                code: "VALIDATION_ERROR",
                message: "bad".into(),
                category: "validation",
            }),
        };
        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["ok"], false);
        assert!(json.get("data").is_none());
        assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
    }
}
