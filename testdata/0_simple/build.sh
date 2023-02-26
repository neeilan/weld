#!/bin/sh

# Use -O0 as -O1 and above introduce unimplemented relocation types
gcc -O0 -c ./*.c