/* Linker script for the nRF9160 in Non-secure mode */
MEMORY
{
    /* NOTE 1 K = 1 KiBi = 1024 bytes */
    SPM                      : ORIGIN = 0x00000000, LENGTH = 320K
    FLASH                    : ORIGIN = 0x00050000, LENGTH = 704K
    RAM                      : ORIGIN = 0x20018000, LENGTH = 160K
}

/* This is commented out after first flash, so we don't have to flash it over and over */
SECTIONS
{
  .spm :
  {
    KEEP(*(.spm .spm.*));
  } > SPM
}
