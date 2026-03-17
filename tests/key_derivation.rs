#[test]
fn known_private_key_derives_expected_address() {
    // Well-known test key (DO NOT use for real funds)
    let key_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let bytes = hex::decode(key_hex).unwrap();
    let signer =
        alloy::signers::local::LocalSigner::from_slice(&bytes).unwrap();
    let address = format!("{}", signer.address());
    // This is hardhat's first default account
    assert_eq!(
        address.to_lowercase(),
        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
    );
}

#[test]
fn address_format_is_checksummed() {
    let key_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let bytes = hex::decode(key_hex).unwrap();
    let signer =
        alloy::signers::local::LocalSigner::from_slice(&bytes).unwrap();
    let address = format!("{}", signer.address());
    // alloy produces checksummed addresses
    assert_eq!(address, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
}

#[test]
fn sign_and_verify_message() {
    use alloy::signers::SignerSync;

    let key_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let bytes = hex::decode(key_hex).unwrap();
    let signer =
        alloy::signers::local::LocalSigner::from_slice(&bytes).unwrap();

    let message = b"hello web3-cli";
    let hash = alloy::primitives::eip191_hash_message(message);
    let sig = signer.sign_hash_sync(&hash).unwrap();

    // Signature should be 65 bytes (r: 32, s: 32, v: 1)
    assert_eq!(sig.as_bytes().len(), 65);
}

#[test]
fn parse_evm_signer_with_0x_prefix() {
    let key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let signer = web3_cli::keys::evm_signer::parse_evm_signer(key);
    assert!(signer.is_ok());
}

#[test]
fn parse_evm_signer_without_prefix() {
    let key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let signer = web3_cli::keys::evm_signer::parse_evm_signer(key);
    assert!(signer.is_ok());
}

#[test]
fn parse_evm_signer_invalid_hex() {
    let key = "not_valid_hex_data_at_all";
    let signer = web3_cli::keys::evm_signer::parse_evm_signer(key);
    assert!(signer.is_err());
}

#[test]
fn parse_evm_signer_wrong_length() {
    let key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478";
    let signer = web3_cli::keys::evm_signer::parse_evm_signer(key);
    assert!(signer.is_err());
}
