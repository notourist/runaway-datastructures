# Code
Der implementierte Bitvektor liegt in [src/runwway_vector.rs](src/runaway_vector.rs)

# Paper
Ist abgegeben oder [hier](paper/paper.pdf).

# Building
Getestet auf Ubuntu 24.04 LTS mit cargo 1.79.0


Installieren mit `apt -y install gcc rustup; rustup install stable; cargo build --release --bin query_reader` und dann in liegt `query_reader` in `target/release`.

Oder `cargo run --release --bin query_reader [eingabe datei] [ausgabe pfad]`