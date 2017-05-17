require 'predictor'

namespace :prediction do

  task :download => :environment do
    DownloadWorker.perform_async
  end

  task :download_sync => :environment do
    DownloadWorker.new.perform
  end

  task :test => [:environment] do
    Predictor.run Rails.root.join('lib', 'data', '20170515', 'gfs_4_20170515_0000_000.grb2').to_s
  end

end
