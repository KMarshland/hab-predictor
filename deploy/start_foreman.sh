#!/bin/bash

echo Running as $(whoami)

#actually start the server
if [ "$RACK_ENV" == "development" ]; then
        /usr/local/bundle/bin/foreman start -f Procfile.dev
    else
        /usr/local/bundle/bin/foreman start -f Procfile
fi