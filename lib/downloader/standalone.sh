#!/usr/bin/env bash

gem install azure-storage -v 0.14.0.preview --pre
gem install activesupport

ruby "${BASH_SOURCE%/*}/standalone.rb"