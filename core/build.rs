use std::fs;
use std::path::Path;
use novax_abi_build::abi::parser::parse_abi_file;
use novax_abi_build::generator::impl_abi_mod::generate_from_abi;

const NON_GENERATED_CONTENT: [&str; 2] = [
    "lib.rs",
    "utils"
];

const NOVAX_PATH_ENV_NAME: &str = "NOVAX_PATH";
const NOVAX_RERUN_ENV_NAME: &str = "NOVAX_RERUN_BUILD_SCRIPT";

fn main() {
    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir_env = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_env).to_path_buf();
    let out_dir_content = fs::read_dir(&out_dir).unwrap();

    for content in out_dir_content.filter_map(|e| e.ok()) {
        if NON_GENERATED_CONTENT.contains(&content.file_name().to_str().unwrap()) {
            continue
        }

        if let Ok(content_type) = content.file_type() {
            if content_type.is_dir() {
                let _ = fs::remove_dir_all(content.path());
            } else if content_type.is_file() {
                let _ = fs::remove_file(content.path());
            }
        }
    }

    let codegen_path_result = std::env::var(NOVAX_PATH_ENV_NAME);
    let is_docs_rs = std::env::var("DOCS_RS").unwrap_or("0".to_string()) == "1";

    let mut lib_file_content = String::from("");

    if let Ok(codegen_path) = &codegen_path_result {
        if !is_docs_rs {
            let abis_path = Path::new(&cargo_manifest_dir).join(codegen_path).join("abis");
            let files_in_abis_path = fs::read_dir(&abis_path).unwrap();
            for file_result in files_in_abis_path {
                let Ok(file) = file_result else { continue };
                let Ok(file_type) = file.file_type() else { continue };
                let Ok(file_name) = file.file_name().into_string() else { continue };

                if file_type.is_file() && file_name.ends_with(".abi.json") {
                    let abi_path = abis_path.join(file_name);
                    let mut abi = parse_abi_file(&abi_path).unwrap();

                    // Backward compatibility.
                    //
                    // In a few versions of the Rust SDK, the upgrade function was an endpoint in the ABI.
                    // It is now under the field upgradeConstructor.
                    abi.endpoints = abi.endpoints.into_iter()
                        .filter(|endpoint| endpoint.name != "upgrade")
                        .collect();

                    let abi_generated_file = generate_from_abi(&abi).unwrap();

                    lib_file_content += &format!("#[allow(missing_docs)]\npub mod {};\n", abi_generated_file.mod_name);

                    fs::write(
                        out_dir.join(abi_generated_file.file_name),
                        abi_generated_file.file_content
                    ).unwrap();
                }
            }
        }
    }

    let lib_path = out_dir.join("generated_lib.rs");

    fs::write(
        lib_path,
        lib_file_content
    ).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    if let Ok(codegen_path) = codegen_path_result {
        println!("cargo:rerun-if-changed={codegen_path}");
    }
    println!("cargo:rerun-if-env-changed={NOVAX_PATH_ENV_NAME}");
    println!("cargo:rerun-if-env-changed={NOVAX_RERUN_ENV_NAME}");
}