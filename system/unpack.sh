#!/bin/bash

J_FI_PREV_SYSTEM_IMG_HASH=$( tail -n 1 history.txt | awk '{split($0,a," "); print a[1]}' )

if [ "$1" = "origin" ]; then
	system="./img/original/system.img"
elif [ "$1" = "prev" ]; then
	system="./img/versions/system-$J_FI_PREV_SYSTEM_IMG_HASH.img"
elif ( [ ${#1} -eq 32 ] && "$(ls ./img/versions | grep -m 1 $1)" != "" ); then
	system="$(ls ./img/versions | grep -m 1 $1)"
else 
	echo "Please specify a image version to unpack"
	exit 1
fi

if [ ! -f $system ]; then
	echo "the version $system you specified does not exists"
	exit 1
fi


sudo modprobe nandsim first_id_byte=0x2c second_id_byte=0xac third_id_byte=0x90 fourth_id_byte=0x15
cat /proc/mtd
sudo flash_erase /dev/mtd0 0 0
sudo ubiformat /dev/mtd0 -f $system -O 2048
sudo modprobe ubi 

sudo ubiattach -p /dev/mtd0 -O 2048

if [ -c "/dev/ubi0_0" ]; then
	sudo mount -t ubifs /dev/ubi0_0 /mnt/jiostore
else
	sudo mount -t ubifs /dev/ubi0 /mnt/jiostore
fi

echo "unpacked successfully!"
exit 0
