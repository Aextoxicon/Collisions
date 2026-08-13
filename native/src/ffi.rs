use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::parse_code as core_parse_code;

// JSON 请求结构体（接收 C# 端传入的序列化数据）
#[derive(serde::Deserialize)]
struct ParseCodeRequest {
    source: String,
    extension: String,
}

// 解析代码，接收 JSON 字符串，返回 JSON 字符串。
//
// # Safety
// `input` 必须是有效的 C 字符串指针（NUL 结尾）。
#[no_mangle]
pub unsafe extern "C" fn parse_code(input: *const c_char) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        // 1. 读取输入 JSON
        let input_str = unsafe {
            if input.is_null() {
                return error_json("Null input pointer");
            }
            match CStr::from_ptr(input).to_str() {
                Ok(s) => s.to_owned(),
                Err(e) => return error_json(&format!("Invalid UTF-8 input: {e}")),
            }
        };

        // 2. 反序列化
        let request: ParseCodeRequest = match serde_json::from_str(&input_str) {
            Ok(r) => r,
            Err(e) => return error_json(&format!("Invalid JSON request: {e}")),
        };

        // 3. 调用核心解析逻辑
        let result = core_parse_code(request.source, request.extension);

        // 4. 序列化结果
        match serde_json::to_string(&result) {
            Ok(json) => CString::new(json)
                .unwrap_or_else(|_| CString::new("{\"error\":\"JSON contains null byte\"}").unwrap())
                .into_raw(),
            Err(e) => error_json(&format!("Serialize error: {e}")),
        }
    });

    match result {
        Ok(ptr) => ptr,
        Err(_) => error_json("Internal panic in parse_code"),
    }
}

// 释放 Rust 侧分配的 C 字符串。
//
// # Safety
// `s` 必须是由 `parse_code` 返回的指针，且只能调用一次。
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// 构造一个 JSON 错误响应，格式：`{ "error": "..." }`。
fn error_json(msg: &str) -> *mut c_char {
    let json = format!("{{\"error\":\"{}\"}}", msg.replace('"', "\\\""));
    CString::new(json)
        .unwrap_or_else(|_| CString::new("{\"error\":\"unknown\"}").unwrap())
        .into_raw()
}