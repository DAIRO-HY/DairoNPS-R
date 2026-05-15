#ifndef LIB_NPC_IOS_H
#define LIB_NPC_IOS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// NPC信息
typedef struct NpcInfo {
    int64_t client_id;
    char *version;
} RustNpcInfo;

// NPC实时状态信息
typedef struct NpcStatus {
    _Bool is_opened;
    _Bool is_running;
    char *connect_msg;
    uint16_t bridge_count;
    uint16_t pool_count;
    uint64_t in_len;
    uint64_t out_len;
} RustNpcStatus;


/**
 * 打开NPC服务
 */
void npc_open(const char *host, int32_t tcp_port, int32_t udp_port, const char *key);

/**
 * 关闭NPC服务
 */
void npc_close();

// 获取NPC信息
RustNpcInfo* npc_get_info(void);

// 释放NPC信息
void npc_free_info(RustNpcInfo* ptr);


// 获取NPC实时状态信息
RustNpcStatus* npc_get_status(void);

// 释放NPC实时状态信息
void npc_free_status(RustNpcStatus* ptr);

#endif