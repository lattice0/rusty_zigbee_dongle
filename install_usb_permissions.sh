#!/bin/bash

FILE_PATH="/etc/udev/rules.d/99-usb.rules"

LINE='SUBSYSTEM=="usb", ATTR{idVendor}=="0451", ATTR{idProduct}=="16ae", GROUP="plugdev", MODE="0666"'

echo $LINE | sudo tee $FILE_PATH > /dev/null

sudo usermod -aG plugdev $USER

sudo udevadm control --reload-rules
sudo udevadm trigger