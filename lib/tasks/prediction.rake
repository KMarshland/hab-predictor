require 'predictor'

namespace :prediction do

  task :download => :environment do
    DownloadWorker.perform_async
  end

  task :download_sync => :environment do
    DownloadWorker.new.perform
  end

  task :test => [:build, :environment] do
    Predictor.hello
  end

end
