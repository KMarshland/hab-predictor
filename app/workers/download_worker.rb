require 'fileutils'
require Rails.root.join('lib', 'grib', 'grib_convert.rb')

class DownloadWorker
  include Sidekiq::Worker

  PREDICTION_MAX_HOURS = 384
  PREDICTION_PERIODS = %w(0000 0600 1200 1800)
  HOUR_RESOLUTION = 3
  THREAD_POOL_SIZE = (ENV['DOWNLOAD_POOL_SIZE'] || 50).to_i

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
  def download(url)
    at = DateTime.strptime(url.split('/').last, '%Y%m%d')
    dir = Rails.root.join('lib', 'data', at.strftime('%Y%m%d'))

    FileUtils::mkdir_p dir

    start = DateTime.now
    total = (PREDICTION_MAX_HOURS/HOUR_RESOLUTION).ceil * PREDICTION_PERIODS.count

    # build the queue of datasets to download
    datasets = Queue.new
    number_completed = 0

    PREDICTION_PERIODS.each do |period|
      (0..PREDICTION_MAX_HOURS).step(HOUR_RESOLUTION).each do |hour_offset|
        datasets << "#{url}gfs_4_#{at.strftime('%Y%m%d')}_#{period}_#{hour_offset.to_s.rjust(3, '0')}.grb2"
      end
    end

    # make a pool to download them
    threads = [THREAD_POOL_SIZE, datasets.size].min
    workers = []

    threads.times do
      workers << Thread.new do
        begin
          while (file_url = datasets.pop(true)).present?
            download_file file_url, dir

            number_completed += 1

            if number_completed % (total / 10).to_i == 0
              percentage = (100*number_completed/total.to_f)
              elapsed = (DateTime.now - start).to_f * 1.day
              remaining = elapsed / (percentage / 100)  - elapsed

              puts "#{percentage.round(1).to_s.rjust(5)}% complete (#{elapsed.round(2)}s elapsed, #{remaining.round(2)}s remaining)"
            end
          end
        rescue ThreadError
        end
      end
    end

    workers.map(&:join)

    # logs!
    elapsed = (DateTime.now - start).to_f * 1.day
    puts "#{elapsed.round(2)}s to download #{url.split('/').last} (#{total} checked)".green

    GribConvert::convert_folder dir, serial: true
  end

  # downloads a specific dataset
  # returns how long it took to download, in seconds or nil if it didn't download
  def download_file(dataset_url, into, debug: false)
    puts "Starting downloading #{dataset_url.split('/').last}" if debug
    start = DateTime.now

    filename = into.join(dataset_url.split('/').last)
    if File.exists?(filename) && File.size(filename) > 1.megabyte
      puts "    #{dataset_url.split('/').last} already exists" if debug
      return
    end


    response = HTTP.get(dataset_url)
    unless response.code == 200
      puts "    #{dataset_url.split('/').last} does not exist (code: #{response.code})" if debug
      return
    end

    body = response.body

    File.open(filename, 'wb') do |file|
      body.each do |data|
        file.write data
      end
    end

    elapsed = (DateTime.now - start).to_f * 1.day
    puts "#{elapsed.round(2)}s to download #{dataset_url.split('/').last}" if debug

    elapsed
  end

end
