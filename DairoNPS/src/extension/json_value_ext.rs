
/// 解析json扩展
pub trait JsonValueExt {
    fn string(&self, path: &str) -> String;
    fn str(&self, path: &str) -> &str;
}
impl JsonValueExt for serde_json::Value {
    fn string(&self, path: &str) -> String {
        self.pointer(path).and_then(serde_json::Value::as_str).unwrap_or_default().to_string()
    }
    fn str(&self, path: &str) -> &str {
        self.pointer(path).and_then(serde_json::Value::as_str).unwrap_or_default()
    }
}