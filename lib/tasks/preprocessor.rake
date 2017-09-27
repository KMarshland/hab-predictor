
namespace :preprocessor do

  task :run, [:url] => :environment do |_t, args|
    if args[:url].present?
      DownloadWorker.new.perform(args[:url])
    else
      StartPreprocessorWorker.new.perform
    end
  end

end
