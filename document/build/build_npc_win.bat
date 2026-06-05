@echo off
cd %USERPROFILE%
set develop_dir=develop
if not exist ./develop (
    mkdir develop
)
cd develop
if not exist ./DairoNPS-R (
    echo "is not exists DairoNPS-R, clone from github"
    git clone https://github.com/DAIRO-HY/DairoNPS-R.git
)
cd DairoNPS-R
cargo clean
cargo +1.93.0 build --release --package DairoNPC