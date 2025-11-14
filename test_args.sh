#!/bin/bash
echo "Testing argument parsing..."
echo "Args:" "$@"
echo "Arg count:" $#
for ((i=1; i<=$#; i++)); do
    echo "Arg $i: ${!i}"
done
