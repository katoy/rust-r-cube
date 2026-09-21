// docs/review-evidence/acdb7f4/probe_native.rs
// ネイティブ環境における wasm-bindgen 依存 API の検証
use cube_studio::{center_parity, is_solved, is_valid};

fn main() {
    println!("=== Probe: Native Error Handling in Wasm-bindgen Functions ===");

    let invalid_state = "INVALID_STATE_STRING_WITH_BAD_LENGTH";

    println!("1. is_valid(\"INVALID_STATE\") をネイティブで実行:");
    let res_valid = is_valid(invalid_state);
    match res_valid {
        Ok(v) => println!("   Ok({})", v),
        Err(e) => {
            println!("   Err(JsValue) を取得");
            let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                format!("{:?}", e)
            }));
            match panic_res {
                Ok(s) => println!("   format!(\"{{:?}}\") 結果: {}", s),
                Err(_) => println!("   => 【パニック確認】format!(\"{{:?}}\") でプロセスが異常終了しました！"),
            }
        }
    }

    println!("\n2. is_solved(\"INVALID_STATE\") をネイティブで実行:");
    let res_solved = is_solved(invalid_state);
    match res_solved {
        Ok(v) => println!("   Ok({})", v),
        Err(e) => {
            println!("   Err(JsValue) を取得");
            let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                format!("{:?}", e)
            }));
            match panic_res {
                Ok(s) => println!("   format!(\"{{:?}}\") 結果: {}", s),
                Err(_) => println!("   => 【パニック確認】format!(\"{{:?}}\") でプロセスが異常終了しました！"),
            }
        }
    }

    println!("\n3. center_parity(\"INVALID_STATE\") をネイティブで実行:");
    let res_parity = center_parity(invalid_state);
    match res_parity {
        Ok(v) => println!("   Ok({})", v),
        Err(e) => {
            println!("   Err(JsValue) を取得");
            let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                format!("{:?}", e)
            }));
            match panic_res {
                Ok(s) => println!("   format!(\"{{:?}}\") 結果: {}", s),
                Err(_) => println!("   => 【パニック確認】format!(\"{{:?}}\") でプロセスが異常終了しました！"),
            }
        }
    }
}
