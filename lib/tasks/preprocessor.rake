
namespace :preprocessor do

  task :run, [:url] => :environment do |_t, args|
    if args[:url].present?
      RunnerWorker.new.perform(args[:url])
    else
      StartPreprocessorWorker.new.perform
    end
  end

  task :import, [:url] => :environment do |_t, args|
    ImportWorker.new.perform(args[:url])
  end

  task :processed => :environment do
    StartPreprocessorWorker.new.list_processed
  end

end
