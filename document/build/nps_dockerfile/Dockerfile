FROM ubuntu:22.04

ARG TARGETARCH

RUN mkdir /app
WORKDIR /app

COPY ./build/$TARGETARCH/DairoNPS /app/DairoNPS

#授权可执行
RUN chmod +x DairoNPS

#启动脚本
ENTRYPOINT /app/DairoNPS
