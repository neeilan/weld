#!/bin/sh

cd testdata/0_simple;
./build.sh;
cd ../..;
rm weld.out; cargo run -p driver testdata/0_simple/*.o ; chmod +x weld.out ; ./weld.out ; echo "$?"
