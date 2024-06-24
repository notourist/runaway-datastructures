#!/bin/bash

echo -e "New run" >> results.txt
for i in {8..34}
do
    for j in {0..10}
    do
        ../target/release/query_reader ./"${i}"/commands.txt ./"${i}"/out.txt >> results.txt
        echo -e " bits=${i}" >> results.txt
    done
done
