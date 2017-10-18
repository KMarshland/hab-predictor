
namespace :preprocessor do

  task :run, [:url] => :environment do |_t, args|
    if args[:url].present?
      RunnerWorker.new.perform(args[:url])
    else
      StartPreprocessorWorker.new.perform
    end
  end

  task :time, [:url] => :environment do |_t, args|
    start_time = DateTime.now

    PreprocessorWorker.new.perform(args[:url])

    end_time = DateTime.now
    total_time = end_time.to_f - start_time.to_f

    puts
    puts "Time elapsed: #{total_time.round(2)}s"
  end

  task :import, [:url] => :environment do |_t, args|
    ImportWorker.new.perform(args[:url])
  end

  task :processed => :environment do
    StartPreprocessorWorker.new.list_processed
  end

end
