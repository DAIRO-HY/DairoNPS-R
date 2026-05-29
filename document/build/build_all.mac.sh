#!/bin/bash

# 进入项目根目录
cd "$(dirname "$0")/../.."

./document/build/build_npc.mac.sh
./document/build/build_nps.mac.sh
./document/build/build_github.sh
./document/build/build_npc_docker.sh
./document/build/build_nps_docker.sh