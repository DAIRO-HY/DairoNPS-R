/// 将数字转换为更友好的数据大小格式
pub trait StringExtension {
    fn md5(&self) -> String;

    /// hello_word -> helloWord
    fn snake_to_pascal(&self) -> String;
}
impl<T: AsRef<str>> StringExtension for T {
    fn md5(&self) -> String {
        let digest = md5::compute(self.as_ref());

        // 转成16进制字符串
        format!("{:x}", digest)
    }

    /// hello_word -> HelloWord
    fn snake_to_pascal(&self) -> String {
        self.as_ref()
            .split('_')
            .filter(|part| !part.is_empty()) // 跳过连续下划线导致的空段
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>()
    }
}

pub trait AsOptStr {
    fn as_opt_str(&self) -> Option<&str>;
}

impl AsOptStr for Option<String> {
    fn as_opt_str(&self) -> Option<&str> {
        self.as_deref()
    }
}

impl AsOptStr for String {
    fn as_opt_str(&self) -> Option<&str> {
        Some(self.as_str())
    }
}