use crate::{application, biz_errorf};
use crate::dao::{channel_dao, forward_dao};
use crate::web::biz_error::BizError;
use sqlx_context::DbContext;

/// 隧道模式:TCP
const MODE_TCP: i64 = 1;

/// 隧道模式:UDP
const MODE_UDP: i64 = 2;

/// 检查TCP端口是否已被占用(系统端口、TCP隧道、端口转发)
/// - exclude_channel_id 排除的隧道id(编辑隧道时排除自己)
/// - exclude_forward_id 排除的端口转发id(编辑端口转发时排除自己)
pub async fn check_tcp_port(
    ctx: &mut DbContext,
    port: i64,
    exclude_channel_id: i64,
    exclude_forward_id: i64,
) -> Result<(), BizError> {
    check_range(port)?;
    let args = &application::ARGS;
    if port == args.web_port as i64 || port == args.tcp_port as i64 {
        return Err(port_error("该端口已被系统占用，请换一个端口。"));
    }
    if channel_dao::count_by_server_port(ctx, port, MODE_TCP, exclude_channel_id).await.map_err(db_error)? > 0 {
        return Err(port_error("该端口已被其他隧道占用，请换一个端口。"));
    }
    if forward_dao::count_by_server_port(ctx, port, exclude_forward_id).await.map_err(db_error)? > 0 {
        return Err(port_error("该端口已被其他端口转发占用，请换一个端口。"));
    }
    Ok(())
}

/// 检查UDP端口是否已被占用(系统端口、UDP隧道)
/// - exclude_channel_id 排除的隧道id(编辑隧道时排除自己)
pub async fn check_udp_port(ctx: &mut DbContext, port: i64, exclude_channel_id: i64) -> Result<(), BizError> {
    check_range(port)?;
    if port == application::ARGS.udp_port as i64 {
        return Err(port_error("该端口已被系统占用，请换一个端口。"));
    }
    if channel_dao::count_by_server_port(ctx, port, MODE_UDP, exclude_channel_id).await.map_err(db_error)? > 0 {
        return Err(port_error("该端口已被其他隧道占用，请换一个端口。"));
    }
    Ok(())
}

/// 检查端口范围是否合法
fn check_range(port: i64) -> Result<(), BizError> {
    if !(1..=65535).contains(&port) {
        return Err(port_error("端口必须在1~65535之间。"));
    }
    Ok(())
}

/// 端口字段的错误信息
fn port_error(msg: &str) -> BizError {
    BizError::field("serverPort", msg)
}

/// 数据库查询失败的错误信息
fn db_error(e: sqlx::Error) -> BizError {
    biz_errorf!("检查端口占用失败:{}", e)
}
