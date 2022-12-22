#!/bin/sh


if [ "$1" = "" ]; then
    echo "Please specify the mount point path"
    exit 1
elif [ "$2" = "" ]; then
    echo "Please write a short description of the changes made"
    exit 1
elif [ ${#2} -gt 64 ]; then
    echo "message length can't be more than 64 characters"
    exit 1
fi

J_FI_PREV_SYSTEM_IMG_HASH=$( tail -n 1 history.txt | awk '{split($0,a," "); print a[1]}' )

mkfs.ubifs -m 2048 -e 126976 -c 1073 -x lzo -f 8 -k r5 -p 1 -l 5 -r $1 jiofi_fs.ubifs
ubinize -p 131072 -m 2048 -O 2048 -s 2048 -x 1 -Q 100310397 -o system.img ubifs_config.ini

bin_hash=$(md5sum system.img | awk '{split($0,a," "); print a[1]}')

if [ "$bin_hash" = "$J_FI_PREV_SYSTEM_IMG_HASH" ]; then
    echo "No changes found"
    rm jiofi_fs.ubifs
    exit 0
fi


echo "$bin_hash $(date) $2" >> history.txt

version="system-$bin_hash.img"

mv system.img ./img/versions/$version

echo "image created $bin_hash"
