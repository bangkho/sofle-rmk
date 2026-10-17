MEMORY
{
  /* NOTE 1 K = 1 KiB = 1024 bytes */
  /* These values correspond to the nRF52840 WITH the Adafruit nRF52 bootloader
     (nice!nano v2 ships it). */
  FLASH : ORIGIN = 0x00001000, LENGTH = 1020K
  RAM : ORIGIN = 0x20000008, LENGTH = 255K

  /* These values correspond to a bare nRF52840 without a bootloader */
  /* FLASH : ORIGIN = 0x00000000, LENGTH = 1024K */
  /* RAM : ORIGIN = 0x20000000, LENGTH = 256K */
}
