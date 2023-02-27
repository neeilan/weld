weld
====

weld is a static ELF linker


Usage
-----
TLDR: `cargo run -p driver <relocatable files>` where one of the relocatables defines the `_start` symbol.

The above command needs to taken with a big tub of salt as weld is under development and has only been tested with one program (testdata/0_simple). My plan is to add progressively more complex programs under `testdata` over time.

In the meantime, one may simply try the `check.sh` script which:
1. compiles the test translation units
2. rebuilds weld
3. invokes weld with the object files from step 1
4. runs the resulting executable


Target Platform
---------------
Portability isn't a stated goal - both weld and the executables it creates are meant to run on x86-64 Linux (System V) targets. The code reflects this intention and avoids implementing unneeded logic like handling 32-bit versions of ELF structures.
 

Feature support
---------------
ELF subtypes : "many relocatables to one executable" case is supported. No support for shared libraries and archives.

Relocations  : Small code model RIP-relative relocations (R_X86_64_PLT32 and R_X86_64_PC32) are handled.
               Test programs build under small code model but I suspect R_X86_64_PC64 support can be trivially added.
               
Outputs      : weld outputs will always have a fixed number of sections. They may be empty, but a section header will
               be present. Outputs have auxiliary info (SHT, .shstrtab) for inspection using readelf, objdump etc.
      

References
----------
While more complex features have not been added yet, the ELF man page [1] was surprisingly thorough enough to guide most of this implementation. Oracle's Linker and Libraries Guide [2] was a useful supplementary resource.

Note: Although I haven't needed to use it much due to [1] and [2] being great, the ELF spec [3] should be considered the final source-of-truth.

[1] https://man7.org/linux/man-pages/man5/elf.5.html
[2] https://docs.oracle.com/cd/E19683-01/816-1386/index.html
[3] https://refspecs.linuxbase.org/elf/elf.pdf
