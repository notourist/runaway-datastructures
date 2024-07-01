apt -y update && apt -y install gcc rustup && rustup install stable && cd .. && cargo build --release && cd benchi && ./generate.sh && ./run.sh
