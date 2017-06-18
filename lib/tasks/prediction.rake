require 'fileutils'
require Rails.root.join('lib', 'grib', 'grib_convert.rb')

namespace :prediction do

  task :download => :environment do
    DownloadWorker.perform_async
  end

  task :download_sync => :environment do
    DownloadWorker.new.perform
  end

  task :reconvert => :environment do
    # delete first old version
    folder_dir = Rails.root.join('lib', 'data', folder.to_s)
    Dir.entries(folder_dir).each do |dir|
      data_dir = folder_dir.join(dir)
      next unless dir =~ /^gfs/ && File.directory?(data_dir)

      Dir.entries(data_dir).each do |subdir|
        dir = data_dir.join(subdir)
        next unless File.directory?(dir)

        puts "Removing #{dir}"
        FileUtils.rm_rf(dir)

        break
      end

      break
    end

    # reconvert
    GribConvert::convert_folder Rails.root.join('lib', 'data', folder.to_s).to_s
  end

  task :reconvert_all => :environment do
    # delete ALL old versions
    Dir.entries(Rails.root.join('lib', 'data', folder.to_s)).each do |dir|
      next unless dir =~ /^gfs/ && File.directory?(Rails.root.join('lib', 'data', folder.to_s, dir))

      puts "Removing #{folder}"
      FileUtils.rm_rf(Rails.root.join('lib', 'data', folder.to_s, dir))
    end

    # reconvert
    GribConvert::convert_folder Rails.root.join('lib', 'data', folder.to_s).to_s
  end

  task :convert => :environment do
    GribConvert::convert_folder Rails.root.join('lib', 'data', folder.to_s).to_s
  end

  task :test => [:environment] do
    puts JSON.pretty_generate(Predictor.predict(
        lat: 36.8491253,
        lon: -121.4342394,
        altitude: 0,
        profile: 'standard',
        time: 1.hour.from_now,
        burst_altitude: 25000,
        ascent_rate: 5,
        descent_rate: 5
    ))
  end

  def folder
    Dir.entries(Rails.root.join('lib', 'data')).select { |n|
      n =~ /^\d+$/
    }.map(&:to_i).max
  end

end
