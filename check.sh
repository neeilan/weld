#!/bin/sh

rm weld.out; cargo run -p driver testdata/0_simple/*.o ; chmod +x weld.out ; ./weld.out ; echo "$?"
