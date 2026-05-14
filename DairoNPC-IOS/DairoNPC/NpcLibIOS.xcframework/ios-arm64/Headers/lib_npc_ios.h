#ifndef LIB_NPC_IOS_H
#define LIB_NPC_IOS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * 两个数字相加
 */
void npc_start(const char *host, int32_t tcp_port, int32_t udp_port, const char *key);

/**
 * 停止服务
 */
void npc_stop();

/**
 * 获取当前桥接数量
 */
int32_t npc_bridge_count();

#endif