require "helix_runtime"

begin
  require "predictor/native"
rescue LoadError
  warn "Unable to load predictor/native. Please run `rake build`"
end
