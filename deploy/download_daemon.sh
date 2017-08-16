#!/usr/bin/env bash

# wait for servers to boot
sleep 10

while true
do
	bundle exec rake prediction:download

	# wait 3 hours
	sleep 10800
done
