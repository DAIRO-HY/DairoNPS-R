#!/bin/bash

# 进入项目根目录
cd "$(dirname "$0")/../.."


#github登录票据
github_token=$GITHUB_TOKEN

#项目名
projectName="DairoNPS-R"

repo="DAIRO-HY/$projectName"
branch="main"

# 获取版本号
#Linux使用
#npc_version=$(grep -oP '(?<=version = ")[^"]+' ./lib_npc/Cargo.toml)

#Mac使用
npc_version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./lib_npc/Cargo.toml)

# 获取版本号
#Linux使用
#nps_version=$(grep -oP '(?<=version = ")[^"]+' ./DairoNPS/Cargo.toml)

#Mac使用
nps_version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./DairoNPS/Cargo.toml)

tag_name="nps_${nps_version}<==>npc_${npc_version}"

#---------------------------------------创建标签----------------------------------------
echo "正在创建标签..."

#删除本地已经存在的标签
git tag -d $tag_name

#删除远程标签
git push origin --delete tag $tag_name

git tag $tag_name
git push origin $tag_name


#查询某个文件的id
get_release_id(){
  release_json=$(curl -L \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer $GITHUB_TOKEN" \
    -H "X-GitHub-Api-Version: 2026-03-10" \
    https://api.github.com/repos/DAIRO-HY/DairoNPS-R/releases)
  id=$(echo "$release_json" | jq -r ".[] | select(.name==\"latest\") | .id")

  #返回值
  echo $id
}

# 创建github的release
create_release(){
  release_message="本次发布版本:$tag_name"
  create_release_api_response=$(curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2026-03-10" \
  https://api.github.com/repos/${repo}/releases \
  -d "{\"tag_name\":\"${tag_name}\",\"target_commitish\":\"main\",\"name\":\"latest\",\"body\":\"Description of the release\",\"draft\":false,\"prerelease\":false,\"generate_release_notes\":false}"
  )
  echo "创建Release结果:${create_release_api_response}" >> push_github_tag.log

  release_id=$(echo "$create_release_api_response" | grep -o '"id": [0-9]*' | head -n 1 | grep -o '[0-9]*')
  echo $release_id
}

# 修改github的release
update_release(){
  release_message="本次发布版本:$tag_name"
  create_release_api_response=$(curl -L \
      -X PATCH \
      -H "Accept: application/vnd.github+json" \
      -H "Authorization: Bearer $GITHUB_TOKEN" \
      -H "X-GitHub-Api-Version: 2026-03-10" \
      https://api.github.com/repos/DAIRO-HY/DairoNPS-R/releases/$1 \
      -d "{\"tag_name\":\"${tag_name}\",\"target_commitish\":\"main\",\"name\":\"latest\",\"body\":\"Description of the release\",\"draft\":false,\"prerelease\":false}")
  echo "修改Release结果:${create_release_api_response}" >> push_github_tag.log
}

#---------------------------------------创建或修改Release:latest----------------------------------
release_id=$(get_release_id)
echo release_id:$release_id
if [ -n "$release_id" ]; then
  echo "正在修改Release:latest..."
  update_release $release_id
else
  echo "正在创建Release:latest..."
  release_id=$(create_release)
fi

#查询某个文件的id
get_asset_id_by_name(){
  assets_json=$(curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2026-03-10" \
  https://api.github.com/repos/${repo}/releases/${release_id}/assets)
  id=$(echo "$assets_json" | jq -r ".[] | select(.name==\"$1\") | .id")

  echo "查询某个文件:$1 的结果:"assets_json >> push_github_tag.log

  #返回值
  echo $id
}

#删除某个文件
delete_asset(){
  echo "准备删除文件ID:$1"
  curl -L \
    -X DELETE \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer $GITHUB_TOKEN" \
    -H "X-GitHub-Api-Version: 2026-03-10" \
    https://api.github.com/repos/${repo}/releases/assets/$1
}

#上传文件
upload_file(){
  filename=$(basename $1)
  echo "正在上传编译好的二进制文件:${filename}..."
  upload_file_api_response=$(curl -L \
      -X POST \
      -H "Accept: application/vnd.github+json" \
      -H "Authorization: Bearer $GITHUB_TOKEN" \
      -H "X-GitHub-Api-Version: 2026-03-10" \
      -H "Content-Type: application/octet-stream" \
      "https://uploads.github.com/repos/${repo}/releases/${release_id}/assets?name=${filename}" \
      --data-binary "@$1")
  echo "上传文件$1结果:${upload_file_api_response}" >> push_github_tag.log
}

# 检查文件是否存在，如果存在则删除后上传
check_and_upload_file(){
  file_name="./build/$1"
  if [ ! -f $file_name ]; then
      echo "文件不存在:$file_name" >> push_github_tag.log
      return
  fi
  asset_id=$(get_asset_id_by_name $1)
  echo asset_id:$asset_id
  if [ -n "$asset_id" ]; then
    echo "正在删除文件$file_name..."
    delete_asset $asset_id
#    echo "github release里的文件${file_name}已存在，无需上传"
    return
  fi
  upload_file $file_name
}

#---------------------------------------上传编译好的二进制文件----------------------------------
check_and_upload_file "DairoNPS-linux-386-$nps_version"
check_and_upload_file "DairoNPS-linux-amd64-$nps_version"
check_and_upload_file "DairoNPS-linux-arm32-v7-$nps_version"
check_and_upload_file "DairoNPS-linux-arm64-$nps_version"
check_and_upload_file "DairoNPS-mac-darwin-aarch64-$nps_version"
check_and_upload_file "DairoNPS-mac-darwin-amd64-$nps_version"

check_and_upload_file "DairoNPC-$npc_version.apk"
check_and_upload_file "DairoNPC-linux-386-$npc_version"
check_and_upload_file "DairoNPC-linux-amd64-$npc_version"
check_and_upload_file "DairoNPC-linux-arm32-v7-$npc_version"
check_and_upload_file "DairoNPC-linux-arm64-$npc_version"
check_and_upload_file "DairoNPC-mac-darwin-aarch64-$npc_version"
check_and_upload_file "DairoNPC-mac-darwin-amd64-$npc_version"