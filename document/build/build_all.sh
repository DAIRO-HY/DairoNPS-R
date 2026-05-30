#!/bin/bash

# 进入项目根目录
cd "$(dirname "$0")/../.."

#./document/build/build_npc.sh
./document/build/build_npc_android.sh
#./document/build/build_nps.sh
./document/build/build_github.sh
#./document/build/build_npc_docker.sh
#./document/build/build_nps_docker.sh