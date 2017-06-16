#! /bin/bash

# Activates the python virtualenv, then the interface server
source '../../../tawhiri/venv/bin/activate'
python 'habmc_interface.py'
deactivate
