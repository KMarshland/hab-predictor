require 'helix_runtime/build_task'

HelixRuntime::BuildTask.new("predictor")

task :default => :build
