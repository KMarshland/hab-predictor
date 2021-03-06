require Rails.root.join('lib', 'storage', 'processed_datasets.rb')

class StartPreprocessorWorker

  PREDICTION_MAX_HOURS = 384
  PREDICTION_PERIODS = %w(0000 0600 1200 1800)
  HOUR_RESOLUTION = 3
  WORKER_POOL_SIZE = (ENV['DOWNLOAD_POOL_SIZE'] || 90).to_i

  def perform(at=DateTime.now)

    at = find_dataset_date at

    # see which ones you've already processed
    @processed_datasets = ProcessedDatasets.on(at)

    puts
    puts 'Has already processed: '
    puts @processed_datasets
    puts

    # download it
    start_preprocessors at
  end

  def list_processed
    at = find_dataset_date DateTime.now
    processed = ProcessedDatasets.on(at)

    at_string = at.strftime('%b %e, %Y')

    puts

    if processed.count == 0
      puts "Has not yet processed anything for the #{at_string} dataset"
    elsif processed.count == 1
      puts "Has already processed #{processed.count} item for the #{at_string} dataset: "
      puts processed
    else
      puts "Has already processed #{processed.count} items for the #{at_string} dataset: "
      puts processed

      puts
      puts "#{processed.count} total"
    end

  end

  private

  def find_dataset_date(at)
    # find the folder the data lives in
    at = at.utc.beginning_of_day

    while true
      url = "https://nomads.ncdc.noaa.gov/data/gfs4/#{at.strftime('%Y%m')}/#{at.strftime('%Y%m%d')}/"

      puts "Checking #{url}"

      break if url_exists? url

      at -= 1.day
    end

    at
  end

  def url_exists?(url)
    sanitized_url = url.gsub('"', '').gsub("\\", '')
    code = `curl -I -s -o /dev/null -w "%{http_code}" "#{sanitized_url}"`.to_i

    # if it's down temporarily, wait a second then retry
    tries = 0
    while code == 503 && tries < 10
      sleep 1
      code = `curl -I -s -o /dev/null -w "%{http_code}" "#{sanitized_url}"`.to_i
      tries += 1
    end

    raise "Unexpected response code: #{code} (#{url})" unless code == 200 || code == 404

    code == 200
  end

  def start_preprocessors(at)
    url = "https://nomads.ncdc.noaa.gov/data/gfs4/#{at.strftime('%Y%m')}/#{at.strftime('%Y%m%d')}"

    puts "Downloading #{url}"
    puts

    start = DateTime.now
    total = (PREDICTION_MAX_HOURS/HOUR_RESOLUTION).ceil * PREDICTION_PERIODS.count

    # build the queue of datasets to download
    datasets = Queue.new
    number_completed = 0

    PREDICTION_PERIODS.each do |period|
      (0..PREDICTION_MAX_HOURS).step(HOUR_RESOLUTION).each do |hour_offset|
        datasets << "#{url}/gfs_4_#{at.strftime('%Y%m%d')}_#{period}_#{hour_offset.to_s.rjust(3, '0')}.grb2"
      end
    end

    # make a pool to download them
    threads = [WORKER_POOL_SIZE, datasets.size].min
    workers = []

    if threads == 1
      while (file_url = datasets.pop(true)).present?
        start_preprocessor file_url

        number_completed += 1

        if number_completed % (total / 10).to_i == 0
          percentage = (100*number_completed/total.to_f)
          elapsed = (DateTime.now - start).to_f * 1.day
          remaining = elapsed / (percentage / 100)  - elapsed

          puts "#{percentage.round(1).to_s.rjust(5)}% complete (#{elapsed.round(2)}s elapsed, #{remaining.round(2)}s remaining)"
        end
      end
    else
      threads.times do
        workers << Thread.new do
          begin
            while (file_url = datasets.pop(true)).present?
              start_preprocessor file_url

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
    end

  end

  def has_processed?(url)
    name = url.split('/').last.split('.').first
    @processed_datasets.include? name
  end

  def start_preprocessor(url)
    return :skipped unless url_exists? url
    return :skipped if has_processed? url

    command = "heroku run:detached rake preprocessor:run[#{url}] --app=hab-prediction"

    puts `#{command}`

  end

end
