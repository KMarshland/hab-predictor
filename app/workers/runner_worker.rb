
class RunnerWorker

  def perform(dataset_url)

    key = "preprocessor[#{dataset_url}]"

    if $redis.exists(key)
      puts "Already running for #{dataset_url}"
      return
    end

    start_time = DateTime.now

    $redis.set(key, true)
    $redis.expire(key, 3.hours)

    DownloadWorker.new.perform(dataset_url)
    PreprocessorWorker.new.perform(dataset_url)
    ZipWorker.new.perform(dataset_url)
    UploadWorker.new.perform(dataset_url)

    $redis.lpush('to_download', dataset_url)

    $redis.del(key)

    end_time = DateTime.now

    total_time = end_time.to_f - start_time.to_f

    puts
    puts "End to end time: #{total_time.round(2)}s"
  rescue Exception => e
    $redis.del(key)
    raise e
  end

end
