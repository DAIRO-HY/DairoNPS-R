#!/bin/bash



# 当前 sh 文件所在目录
script_dir="$(cd "$(dirname "$0")" && pwd)"

# 项目目录
project_root="$(cd "$script_dir/../.." && pwd)"

cd /root/develop_tool
ls
echo "-------------------"
pwd
echo "-------------------"

#./document/build/build_npc.sh
#./document/build/build_npc_android.sh
#./document/build/build_nps.sh
#./document/build/build_github.sh
#./document/build/build_npc_docker.sh
#./document/build/build_nps_docker.sh