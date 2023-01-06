#!/bin/sh

 
# write log file if executable throws something at stdout/sterr
exec >> /usr/www/en-jio/executable.log 2>&1
# execute the binary, should run in foreground, otherwise get in loop
echo "$(date): Starting program..."
exec /usr/sbin/server

