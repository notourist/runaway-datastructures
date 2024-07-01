apt -y update && apt -y install gcc rustup && rustup install stable && cargo build --release && ./generate.sh && ./run.sh
