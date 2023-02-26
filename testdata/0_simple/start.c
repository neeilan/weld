#include <sys/syscall.h>

int main(void);

// In the System V AMD-64 ABI, the first integer arg in
// system calls is passed in register %rdi,
// The syscall number goes in %rax.
void call_exit(int code, int exit_syscall_num) {
	// Pure register usage based on ABI breaks at -O2 or -O3 so
	// we use the local variables. At -O0, we don't need to use
	// the variables, and can copy the right values reg-to-reg
	// in asm alone.
	asm("mov %0, %%eax;"   // Copy syscall number into %rax
	    "mov %1, %%edi;"   // Copy main's return value into rdi
		"syscall;" : /**no outputs*/ : "r"(exit_syscall_num), "r"(code));
}

void _start() {
    call_exit(main(), SYS_exit);
}

