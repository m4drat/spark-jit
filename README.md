# Spark JIT

This project originally started as an unrealized CTF task. I wanted to write a custom JIT compiler for a simple language. The challenge has since been abandoned (mostly because I started to work on it a day before the CTF 😅), but the core of it - the JIT compiler - has been extracted and is now being developed as a standalone project.

The goal of this project is to practice writing some Rust code as well as to study instrution encodings, and of course to reinvent the wheel :)

## References

1. A lot of inspiration was taken from this great project: [juicebox-asm](https://github.com/johannst/juicebox-asm).
2. As a reference implementation I used [LibJIT](https://github.com/SerenityOS/serenity/tree/master/Userland/Libraries/LibJIT) from SerenityOS.
3. As instructions encoding reference I used [felixcloutier.com/x86](https://www.felixcloutier.com/x86/), and [Intel Software Developer Manuals](https://software.intel.com/en-us/download/intel-64-and-ia-32-architectures-sdm-combined-volumes-1-2a-2b-2c-2d-3a-3b-3c-3d-and-4).
