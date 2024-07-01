apt -y update && apt -y upgrade && apt -y install gcc rustup && rustup install default && cargo build --release && ./generate.sh && ./run.sh
