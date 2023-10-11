use std::collections::HashMap;
use std::env;
use std::io::prelude::*;

use flate2::read::ZlibDecoder;
use serde::{de, Deserialize, Serialize};
use serde_json::json;

fn check_large_dataframe_for_nulls(partition_key: &str) -> u32 {
    // smoke and mirrors
    return 1;
}

#[derive(Debug)]
struct PipesContext {
    partition_key: String,
}

fn decode_env_var<T>(param: &str) -> T
{
    let zlib_compressed_slice = base64::decode(param).unwrap();
    let mut decoder = ZlibDecoder::new(&zlib_compressed_slice[..]);
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str).unwrap();
    let value: T = serde_json::from_str(&json_str).unwrap();
    return value;
}

#[derive(Debug, Serialize)]
struct PipesMessage {
    __dagster_pipes_version: String,
    method: String,
    params: Option<HashMap<String, serde_json::Value>>,
}

fn report_asset_check(
    context: &mut PipesContext,
    check_name: &str,
    passed: bool,
    asset_key: &str,
    metadata: serde_json::Value,
) {
    let params: HashMap<String, serde_json::Value> = HashMap::from([
        ("asset_key".to_string(), json!(asset_key)),
        ("check_name".to_string(), json!(check_name)),
        ("passed".to_string(), json!(passed)),
        ("severity".to_string(), json!("ERROR")), // hardcode for now
        ("metadata".to_string(), metadata),
    ]);

    let msg = PipesMessage {
        __dagster_pipes_version: "0.1".to_string(),
        method: "report_asset_check".to_string(),
        params: Some(params),
    };
    let serialized_msg = serde_json::to_string(&msg).unwrap();
    eprintln!("{}", serialized_msg);
}

fn main() {
    let mut context = decode_env_var<PipesContext>(env::var("DAGSTER_PIPES_CONTEXT"))
    let null_count = check_large_dataframe_for_nulls(context.partition_key);
    let passed = null_count == 0;
    let metadata = json!({"null_count": {"raw_value": null_count, "type": "int"}});

    report_asset_check(
        context,
        "telem_post_processing_check",
        passed,
        "telem_post_processing",
        metadata,
    );
}
