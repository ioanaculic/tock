MEMORY
{
  rom (rx)  : ORIGIN = 0x00010000, LENGTH = 128K
  prog (rx) : ORIGIN = 0x00030000, LENGTH = 128K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 32K
}

MPU_MIN_ALIGN = 1K;
PAGE_SIZE = 4K;

INCLUDE ../kernel_layout.ld
