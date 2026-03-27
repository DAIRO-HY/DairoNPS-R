use chrono::DateTime;

/// 将数字转换为更友好的数据大小格式
pub trait ToDataSize {
    fn data_size(&self) -> String;
}
pub trait NumberDataSize: Copy + TryInto<u64> {}

impl NumberDataSize for i8 {}
impl NumberDataSize for i16 {}
impl NumberDataSize for i32 {}
impl NumberDataSize for i64 {}
impl NumberDataSize for i128 {}
impl NumberDataSize for u8 {}
impl NumberDataSize for u16 {}
impl NumberDataSize for u32 {}
impl NumberDataSize for u64 {}
impl NumberDataSize for u128 {}
impl NumberDataSize for usize {}

impl<T> ToDataSize for T
where
    T: NumberDataSize,
{
    fn data_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        let value = (*self).try_into().unwrap_or(0);
        match value {
            b if b < KB => format!("{} Byte", b),
            b if b < MB => format!("{:.2} KB", b as f64 / KB as f64),
            b if b < GB => format!("{:.2} MB", b as f64 / MB as f64),
            b if b < TB => format!("{:.2} GB", b as f64 / GB as f64),
            b => format!("{:.2} TB", b as f64 / TB as f64),
        }
    }
}

/// 将时间戳转换为日期格式
pub trait ToDateFormat {
    /// 将时间戳毫秒转换为日期格式
    fn date_format(&self) -> String;
}

pub trait IntegerTimestamp: Copy + TryInto<i64> {}

impl IntegerTimestamp for i32 {}
impl IntegerTimestamp for i64 {}
impl IntegerTimestamp for i128 {}
impl IntegerTimestamp for u32 {}
impl IntegerTimestamp for u64 {}
impl IntegerTimestamp for u128 {}
impl IntegerTimestamp for usize {}

impl<T> ToDateFormat for T
where
    T: IntegerTimestamp,
{
    fn date_format(&self) -> String {
        let timestamp = (*self).try_into().unwrap_or(0);
        if timestamp == 0 {
            return String::new();
        }
        let dt = DateTime::from_timestamp_millis(timestamp).unwrap();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
