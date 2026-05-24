FROM ubuntu:22.04

ARG TARGETARCH

RUN mkdir /app
WORKDIR /app

#下载程序
#ADD https://github.com/DAIRO-HY/DairoNPC/releases/download/1.0.0/dairo-npc-linux-amd64 /app
COPY ./build/$TARGETARCH/DairoNPC-linux-$TARGETARCH /app/DairoNPC

#授权可执行
RUN chmod +x DairoNPC

# 定义运行时的环境变量
ENV NPC_HOST=127.0.0.1
ENV NPC_KEY=xxxxxxxxx
ENV NPC_TCP=1781
ENV NPC_UDP=1782

#启动脚本
#ENTRYPOINT /app/DairoNPC-linux-amd64 -h:$NPC_HOST -k:$NPC_KEY -t:$NPC_TCP -u:$NPC_UDP
ENTRYPOINT ls
