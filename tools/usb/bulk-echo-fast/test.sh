#!/bin/sh

dd if=/dev/urandom of=input.dat bs=1 count=100000

time ./a.out <input.dat >output.dat

diff input.dat output.dat
