use ethers::prelude::*;
use std::env;
use std::path::PathBuf;

const CONTRACTS: &[(&str, &str)] = &[
    (
        "./abi/Account.json",
        "Account",
    ),
    (
        "./abi/Beacon.json",
        "Beacon",
    ),
    (
        "./abi/Treasury.json",
        "Treasury",
    ),
    (
        "./abi/LiquidityPool.json",
        "LiquidityPool",
    ),
    (
        "./abi/IERC20.json",
        "IERC20",
    ),
    ("./abi/RouterHpsmSynths.json", "RouterHpsmSynths"),
];

fn main() {
    for (path, name) in CONTRACTS {
        generate_contract_code_from_abi(path, name).unwrap();
    }
}

fn generate_contract_code_from_abi(
    relative_abi_path: &str,
    contract_name: &str,
) -> eyre::Result<()> {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let abi_path = PathBuf::from(&crate_dir).join(format!("{relative_abi_path}"));
    let contracts_path = PathBuf::from(out_dir);
    let out_path = contracts_path.join(format!("{contract_name}.rs"));
    if out_path.exists() {
        std::fs::remove_file(&out_path)?;
    }
    Abigen::new(contract_name, abi_path.to_str().unwrap())?
        .generate()?
        .write_to_file(&out_path)?;
    Ok(())
}
