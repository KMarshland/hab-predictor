#!/bin/bash

echo Deploying as $(whoami)

service nginx start
service cron start
whenever -w

# Start the server itself with lower permissions
su - run -p -c 'bash ./deploy/start_puma.sh'