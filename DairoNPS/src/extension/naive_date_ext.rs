use chrono::{Datelike, NaiveDate, NaiveDateTime, ParseResult};

/// 将日期格式化
pub trait NaiveDateTimeExt {
    fn ymdhms(&self) -> String;
    fn ymd(&self) -> String;
    fn from_ymd(str: &str) -> ParseResult<NaiveDateTime>;
    fn from_ymdhms(str: &str) -> ParseResult<NaiveDateTime>;
    fn age(&self) -> String;
    fn days_from_today(&self) -> i64;
}
impl NaiveDateTimeExt for NaiveDateTime {

    /// 格式化输出: yyyy-MM-dd HH:mm:ss
    fn ymdhms(&self) -> String {
        self.format("%Y-%m-%d %H:%M:%S").to_string()
    }


    /// 格式化输出: yyyy-MM-dd
    fn ymd(&self) -> String {
        self.format("%Y-%m-%d").to_string()
    }
    fn from_ymd(str: &str) -> ParseResult<NaiveDateTime>{
        NaiveDate::parse_from_str(str, "%Y-%m-%d")
            .map(|date| date.and_hms_opt(0, 0, 0).unwrap())
    }

    fn from_ymdhms(str: &str) -> ParseResult<NaiveDateTime>{
        NaiveDateTime::parse_from_str(
            str,
            "%Y-%m-%d %H:%M:%S",
        )
    }

    /// 获取年龄（年+月）
    fn age(&self) -> String {
        // 检查是否为零值（年份为1，月日为1）
        if self.year() == 1 {
            return String::new();
        }

        let now = chrono::Local::now().naive_local();

        let mut years = now.year() - self.year();
        let mut months = now.month() as i32 - self.month() as i32;

        // 处理当前月还没到生日那天的情况
        if now.day() < self.day() {
            months -= 1; // 当前月生日还没过，少一个月
        }

        // 如果月份是负数，说明今年还没过生日月
        if months < 0 {
            years -= 1;
            months += 12;
        }
        format!("{}岁{}个月", years, months)
    }

    /// 计算当前日期距离今天的天数（未来为正，过去为负）
    fn days_from_today(&self) -> i64 {
        let today = chrono::Local::now().naive_local().date();
        (self.date() - today).num_days()
    }

}