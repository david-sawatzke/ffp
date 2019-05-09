#!/usr/bin/env python3
"""
Set FFP LED
"""

import sys
import usb.core


ID_VENDOR = 0x1209
ID_PRODUCT = 0x0001
SET_LED = 6


def main():
    dev = usb.core.find(idVendor=ID_VENDOR, idProduct=ID_PRODUCT)
    dev.ctrl_transfer(
        bmRequestType=2, bRequest=SET_LED, wValue=int(sys.argv[1]))


if __name__ == "__main__":
    main()