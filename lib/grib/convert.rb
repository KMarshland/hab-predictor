require_relative './grib_convert.rb'

GribConvert::convert ARGV.last unless ARGV.size == 0 || ARGV.last.empty?

