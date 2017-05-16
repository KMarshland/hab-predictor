require 'fileutils'

class DownloadWorker
  include Sidekiq::Worker

  # downloads the dataset for the given day (will look in the past if it can't find it)
  def perform(at=DateTime.now)

    # find the folder the data lives in

    at = at.utc.beginning_of_day
    url = nil

    while true
      url = "https://nomads.ncdc.noaa.gov/data/gfs4/#{at.strftime('%Y%m')}/#{at.strftime('%Y%m%d')}/"
      break if dataset_exists? url

      at -= 1.day
    end

    # download it
    download url
  end

  def dataset_exists?(url)
    code = HTTP.get(url).code

    raise "Unexpected response code: #{code}" unless code == 200 || code == 404

    code == 200
  end

  # downloads a dataset from a folder definition
  PREDICTION_MAX_HOURS = 384
  HOUR_RESOLUTION = 3
  def download(url)
    at = DateTime.strptime(url.split('/').last, '%Y%m%d')
    dir = Rails.root.join('lib', 'data', at.strftime('%Y%m%d'))

    FileUtils::mkdir_p dir

    start = DateTime.now
    threads = []
    number_completed = 0

    (0..PREDICTION_MAX_HOURS).step(HOUR_RESOLUTION).each do |hour_offset|
      threads << Thread.new("#{url}gfs_4_#{at.strftime('%Y%m%d')}_0000_#{hour_offset.to_s.rjust(3, '0')}.grb2") do |file_url|
        download_file file_url, dir

        number_completed += 1
        percentage = (100*number_completed/PREDICTION_MAX_HOURS.to_f/HOUR_RESOLUTION)
        elapsed = (DateTime.now - start).to_f * 1.day
        puts "#{percentage.round(1)}% complete (#{elapsed}s elapsed, #{0.01 * elapsed * (100 - percentage)}s remaining)" if hour_offset % 15 == 0
      end
    end

    threads.map(&:join)

    elapsed = (DateTime.now - start).to_f * 1.day
    puts "#{elapsed}s to download #{url.split('/').last}".green
  end

  # downloads a specific dataset
  def download_file(dataset_url, into, debug: false)
    start = DateTime.now

    body = HTTP.get(dataset_url).body

    File.open(into.join(dataset_url.split('/').last), 'wb') do |file|
      body.each do |data|
        file.write data
      end
    end

    elapsed = (DateTime.now - start).to_f * 1.day
    puts "#{elapsed}s to download #{dataset_url.split('/').last}" if debug
  end

end
