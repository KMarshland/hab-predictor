
is_rake = !ARGV.any?{ |arg|
  arg =~ /puma/
}

unless is_rake

  Process.fork do
    loop do

      begin
        length = $redis.llen('to_download')

        puts "\t [dataset downloader] #{length} items to download"

        if length == 0
          sleep 1.minute
          next
        end

        list = $redis.lrange('to_download', 0, length - 1)
        list.each do |item|
          ImportWorker.new.perform item
        end
      rescue Redis::CannotConnectError
        puts 'Could not connect to redis'
      end

      sleep 1.minute

    end
  end

end
