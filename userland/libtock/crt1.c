#include <tock.h>

extern unsigned int* _etext;
extern unsigned int* _edata;
extern unsigned int* _got;
extern unsigned int* _egot;
extern unsigned int* _bss;
extern unsigned int* _ebss;
extern int main();

__attribute__ ((section(".start"), used))
void _start(
    __attribute__((unused))void* mem_start,
    __attribute__((unused))void* app_memory_break,
    __attribute__((unused))void* kernel_memory_break) {
  main();
  while(1) { yield(); }
}

