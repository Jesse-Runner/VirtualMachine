#!/bin/bash

ERR=0
INPUTS=`ls *.o`

for f in $INPUTS;
do
    ../target/release/vm $f > "${f%.o}.student"
    if ! diff -q "${f%.o}.student" "${f%.o}.expected" &>/dev/null; then
	printf "%-10s %10s\n" $f "ERROR, outputs differ"
	ERR=1
    else
	printf "%-10s %10s\n" $f "passed"
    fi
done

exit $ERR
