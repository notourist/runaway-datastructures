#!/bin/bash

for i in {8..34}
do
    vecBlocks64=$((2**(i-6)))
    mkdir ./"${i}"
   generator -l ${vecBlocks64} -c 1000000 -o ./"${i}"
done

