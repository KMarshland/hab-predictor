#!/bin/bash

echo Running as $(whoami)

# will make sessions expire with every deploy
export SECRET_KEY_BASE=$(rake secret)

#actually start the server
if [ "$RACK_ENV" == "development" ]; then
        /usr/local/bundle/bin/foreman start -f Procfile.dev
    else
        /usr/local/bundle/bin/foreman start -f Procfile
fi
