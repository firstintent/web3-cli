use assert_cmd::Command;
use predicates::prelude::*;

fn web3() -> Command {
    let mut cmd = Command::cargo_bin("web3").unwrap();
    // Isolate each test from shared config by using a temp dir.
    // Tests that need config isolation should use web3_isolated() instead.
    cmd
}

fn web3_isolated() -> (Command, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("web3").unwrap();
    cmd.env("XDG_CONFIG_HOME", tmp.path());
    (cmd, tmp)
}

#[test]
fn help_succeeds() {
    web3().arg("--help").assert().success().stdout(
        predicate::str::contains("Multi-chain Web3 wallet CLI")
            .and(predicate::str::contains("wallet"))
            .and(predicate::str::contains("balance"))
            .and(predicate::str::contains("send"))
            .and(predicate::str::contains("sign")),
    );
}

#[test]
fn version_succeeds() {
    web3()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("web3"));
}

#[test]
fn wallet_help() {
    web3()
        .args(["wallet", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("create")
                .and(predicate::str::contains("import-key"))
                .and(predicate::str::contains("show"))
                .and(predicate::str::contains("reset")),
        );
}

#[test]
fn chain_list_json() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args(["--output", "json", "chain", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("ethereum")
                .and(predicate::str::contains("polygon"))
                .and(predicate::str::contains("solana"))
                .and(predicate::str::contains("sui")),
        );
}

#[test]
fn chain_list_table() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args(["chain", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ethereum"));
}

#[test]
fn chain_info_json() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args(["--output", "json", "chain", "info"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"ok\": true")
                .and(predicate::str::contains("ethereum"))
                .and(predicate::str::contains("rpc_url")),
        );
}

#[test]
fn validate_evm_address_json() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args([
        "--output",
        "json",
        "validate",
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    ])
    .assert()
    .success()
    .stdout(
        predicate::str::contains("\"valid\": true")
            .and(predicate::str::contains("ethereum"))
            .and(predicate::str::contains("polygon")),
    );
}

#[test]
fn validate_invalid_address() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args(["--output", "json", "validate", "not_an_address"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"valid\": false"));
}

#[test]
fn config_path() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("web3-cli"));
}

#[test]
fn config_show_json() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args(["--output", "json", "config", "show"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("default_chain")
                .and(predicate::str::contains("chains")),
        );
}

#[test]
fn wallet_show_no_wallet_exits_nonzero() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args(["--output", "json", "wallet", "show"])
        .env_remove("WEB3_PRIVATE_KEY")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"ok\": false"));
}

#[test]
fn send_no_wallet_errors() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args([
        "--output",
        "json",
        "send",
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        "0.1",
    ])
    .env_remove("WEB3_PRIVATE_KEY")
    .assert()
    .failure();
}

#[test]
fn chain_set_rejects_http_rpc() {
    let (mut cmd, _tmp) = web3_isolated();
    cmd.args([
        "chain",
        "set",
        "ethereum",
        "--rpc",
        "http://insecure.example.com",
    ])
    .assert()
    .failure();
}

#[test]
fn evm_subcommand_help() {
    web3()
        .args(["evm", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("call")
                .and(predicate::str::contains("send"))
                .and(predicate::str::contains("token")),
        );
}

#[test]
fn solana_subcommand_help() {
    web3()
        .args(["solana", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("invoke"));
}

#[test]
fn sui_subcommand_help() {
    web3()
        .args(["sui", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("move-call"));
}

// --- JSON Envelope Schema Tests ---
// These lock down the JSON output contract that AI agents depend on.

#[test]
fn json_envelope_success_schema() {
    let (mut cmd, _tmp) = web3_isolated();
    let output = cmd
        .args(["--output", "json", "chain", "info"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    // Required fields for success envelope
    assert_eq!(json["ok"], true, "success envelope must have ok=true");
    assert!(json.get("data").is_some(), "success envelope must have data field");

    // Error field must be absent on success
    assert!(
        json.get("error").is_none(),
        "success envelope must not have error field"
    );
}

#[test]
fn json_envelope_error_schema() {
    let (mut cmd, _tmp) = web3_isolated();
    let output = cmd
        .args(["--output", "json", "wallet", "show"])
        .env_remove("WEB3_PRIVATE_KEY")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    // Required fields for error envelope
    assert_eq!(json["ok"], false, "error envelope must have ok=false");
    assert!(json.get("error").is_some(), "error envelope must have error field");

    // Error object structure
    let error = &json["error"];
    assert!(error.get("code").is_some(), "error must have code field");
    assert!(error.get("message").is_some(), "error must have message field");
    assert!(error.get("category").is_some(), "error must have category field");

    // Data field must be absent on error
    assert!(
        json.get("data").is_none(),
        "error envelope must not have data field"
    );
}

#[test]
fn json_envelope_chain_fields_present() {
    let (mut cmd, _tmp) = web3_isolated();
    let output = cmd
        .args(["--output", "json", "chain", "info"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    // chain info includes chain/network context
    assert!(
        json.get("chain").is_some() || json.get("data").is_some(),
        "chain-scoped commands should include chain context"
    );
}

#[test]
fn json_output_is_valid_json_for_all_commands() {
    // Verify that --output json always produces valid JSON, even on errors
    let commands: Vec<Vec<&str>> = vec![
        vec!["--output", "json", "chain", "list"],
        vec!["--output", "json", "chain", "info"],
        vec!["--output", "json", "config", "show"],
        vec!["--output", "json", "validate", "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"],
        vec!["--output", "json", "validate", "invalid"],
    ];

    for args in &commands {
        let (mut cmd, _tmp) = web3_isolated();
        let output = cmd.args(args).output().unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&output.stdout);
        assert!(
            parsed.is_ok(),
            "Command {:?} did not produce valid JSON: {}",
            args,
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

// --- Wallet Roundtrip Tests ---
// These exercise the full encrypt → save → load → decrypt → derive chain.

#[test]
fn wallet_create_then_show_roundtrip() {
    let (mut cmd, tmp) = web3_isolated();

    // Step 1: Create wallet
    let create_output = cmd
        .args(["--output", "json", "wallet", "create"])
        .output()
        .unwrap();
    assert!(
        create_output.status.success(),
        "wallet create failed: {}",
        String::from_utf8_lossy(&create_output.stdout)
    );
    let create_json: serde_json::Value =
        serde_json::from_slice(&create_output.stdout).unwrap();
    assert_eq!(create_json["ok"], true);
    let created_address = create_json["data"]["evm_address"]
        .as_str()
        .expect("evm_address must be a string");
    assert!(
        created_address.starts_with("0x") && created_address.len() == 42,
        "Address must be 0x + 40 hex chars, got: {created_address}"
    );

    // Step 2: Show wallet — should return the same address
    let (mut cmd2, _) = (
        {
            let mut c = Command::cargo_bin("web3").unwrap();
            c.env("XDG_CONFIG_HOME", tmp.path());
            c
        },
        &tmp,
    );
    let show_output = cmd2
        .args(["--output", "json", "wallet", "show"])
        .env_remove("WEB3_PRIVATE_KEY")
        .output()
        .unwrap();
    assert!(
        show_output.status.success(),
        "wallet show failed: {}",
        String::from_utf8_lossy(&show_output.stdout)
    );
    let show_json: serde_json::Value =
        serde_json::from_slice(&show_output.stdout).unwrap();
    assert_eq!(show_json["ok"], true);
    assert_eq!(
        show_json["data"]["evm_address"].as_str().unwrap(),
        created_address,
        "wallet show must return the same address as wallet create"
    );
    assert_eq!(show_json["data"]["has_wallet"], true);

    // Step 3: Addresses command
    let mut cmd3 = Command::cargo_bin("web3").unwrap();
    cmd3.env("XDG_CONFIG_HOME", tmp.path());
    let addr_output = cmd3
        .args(["--output", "json", "wallet", "addresses"])
        .env_remove("WEB3_PRIVATE_KEY")
        .output()
        .unwrap();
    assert!(addr_output.status.success());
    let addr_json: serde_json::Value =
        serde_json::from_slice(&addr_output.stdout).unwrap();
    assert_eq!(
        addr_json["data"]["evm"].as_str().unwrap(),
        created_address
    );
}

#[test]
fn wallet_create_then_sign_message() {
    let (mut cmd, tmp) = web3_isolated();

    // Create wallet
    let create_output = cmd
        .args(["--output", "json", "wallet", "create"])
        .output()
        .unwrap();
    assert!(create_output.status.success());

    // Sign a message
    let mut cmd2 = Command::cargo_bin("web3").unwrap();
    cmd2.env("XDG_CONFIG_HOME", tmp.path());
    let sign_output = cmd2
        .args(["--output", "json", "sign", "message", "hello web3-cli"])
        .env_remove("WEB3_PRIVATE_KEY")
        .output()
        .unwrap();
    assert!(
        sign_output.status.success(),
        "sign failed: {}",
        String::from_utf8_lossy(&sign_output.stdout)
    );
    let sign_json: serde_json::Value =
        serde_json::from_slice(&sign_output.stdout).unwrap();
    assert_eq!(sign_json["ok"], true);

    let sig = sign_json["data"]["signature"].as_str().unwrap();
    assert!(sig.starts_with("0x"), "Signature must start with 0x");
    assert_eq!(sig.len(), 132, "EIP-191 signature must be 65 bytes (132 hex chars + 0x)");

    let addr = sign_json["data"]["address"].as_str().unwrap();
    assert!(addr.starts_with("0x") && addr.len() == 42);
}

#[test]
fn wallet_create_force_overwrites() {
    let (mut cmd, tmp) = web3_isolated();

    // Create first wallet
    cmd.args(["--output", "json", "wallet", "create"])
        .assert()
        .success();

    // Create second wallet with --force
    let mut cmd2 = Command::cargo_bin("web3").unwrap();
    cmd2.env("XDG_CONFIG_HOME", tmp.path());
    cmd2.args(["--output", "json", "wallet", "create", "--force"])
        .assert()
        .success();
}

#[test]
fn wallet_create_without_force_fails_if_exists() {
    let (mut cmd, tmp) = web3_isolated();

    // Create first wallet
    cmd.args(["--output", "json", "wallet", "create"])
        .assert()
        .success();

    // Try to create without --force — should fail
    let mut cmd2 = Command::cargo_bin("web3").unwrap();
    cmd2.env("XDG_CONFIG_HOME", tmp.path());
    cmd2.args(["--output", "json", "wallet", "create"])
        .assert()
        .failure();
}

#[test]
fn wallet_reset_then_show_fails() {
    let (mut cmd, tmp) = web3_isolated();

    // Create wallet
    cmd.args(["wallet", "create"]).assert().success();

    // Reset with --force
    let mut cmd2 = Command::cargo_bin("web3").unwrap();
    cmd2.env("XDG_CONFIG_HOME", tmp.path());
    cmd2.args(["wallet", "reset", "--force"]).assert().success();

    // Show should fail
    let mut cmd3 = Command::cargo_bin("web3").unwrap();
    cmd3.env("XDG_CONFIG_HOME", tmp.path());
    cmd3.args(["--output", "json", "wallet", "show"])
        .env_remove("WEB3_PRIVATE_KEY")
        .assert()
        .failure();
}

#[test]
fn chain_set_valid_https_persists() {
    let (mut cmd, tmp) = web3_isolated();

    // Set chain to polygon
    cmd.args(["chain", "set", "polygon"]).assert().success();

    // Verify chain info shows polygon
    let mut cmd2 = Command::cargo_bin("web3").unwrap();
    cmd2.env("XDG_CONFIG_HOME", tmp.path());
    let output = cmd2
        .args(["--output", "json", "chain", "info"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["data"]["name"], "polygon");
}
