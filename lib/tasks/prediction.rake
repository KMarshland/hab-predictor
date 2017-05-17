
namespace :prediction do

  task :download => :environment do
    DownloadWorker.perform_async
  end

  task :download_sync => :environment do
    DownloadWorker.new.perform
  end

  task :test => [:environment] do
    Predictor.predict(
                 lat: 36.8491253,
                 lon: -121.4342394,
                 altitude: 0,
                 profile: 'standard',
                 time: 1.hour.from_now,
                 burst_altitude: 25000,
                 ascent_rate: 5,
                 descent_rate: 5
    )
  end

end
