FROM ubuntu:22.04

ARG TARGETARCH

RUN mkdir /app
WORKDIR /app

COPY ./build/$TARGETARCH/DairoNPC /app/DairoNPC

#授权可执行
RUN chmod +x DairoNPC

#启动脚本
ENTRYPOINT exec /app/DairoNPC
