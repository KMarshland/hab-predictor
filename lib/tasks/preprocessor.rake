
namespace :preprocessor do

  task :run => :environment do
    StartPreprocessorWorker.new.perform
  end

end
