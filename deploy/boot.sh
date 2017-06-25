#!/bin/bash

echo Deploying as $(whoami)

sed -i "s/80/${PORT:-80}/" /etc/nginx/nginx.conf

service nginx start
service cron start
whenever -w

# rotate secret
# This may cause problems if we ever do things with cookies
export SECRET_KEY_BASE=$(rake secret)

# Start the server itself with lower permissions
su - run -p -c 'bash ./deploy/start_puma.sh'