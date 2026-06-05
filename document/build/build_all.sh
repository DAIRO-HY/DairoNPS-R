#!/bin/bash



# 当前 sh 文件所在目录
script_dir="$(cd "$(dirname "$0")" && pwd)"

# 项目目录
project_root="$(cd "$script_dir/../.." && pwd)"

#cd /root/develop_tool
ls
echo "-------------------"
pwd
echo "-------------------"


docker run --platform linux/amd64 --rm -v /home/docker/jenkins+jdk17+docker/data/develop_tool/project/DairoNPS-R:/home/rust/src -w /home/rust/src rust:1.93 \
sh -c "ls"

#./document/build/build_npc.sh
#./document/build/build_npc_android.sh
#./document/build/build_nps.sh
#./document/build/build_github.sh
#./document/build/build_npc_docker.sh
#./document/build/build_nps_docker.sh