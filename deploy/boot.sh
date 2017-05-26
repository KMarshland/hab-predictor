#!/bin/bash

echo Deploying as $(whoami)

service nginx start
service cron start
service ssh start
whenever -w

#Start the server itself
su - run -p -c 'bash ./deploy/start_puma.sh'