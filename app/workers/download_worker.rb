require 'open3'
require 'fileutils'

class DownloadWorker

  # Downloads the given dataset from the remote
  def perform(dataset_url)

    @dataset_url = dataset_url

    FileUtils::mkdir_p Rails.root.join('data')

    puts "Downloading #{@dataset_url}"

    @output_path = Rails.root.join('data', @dataset_url.split('/').pop)

    @partial_name = "#{@output_path.to_s.split('.').first}.grb2.partial"

    puts "Downloading #{remote_bytes} bytes"

    Open3.popen3('curl', '-o', @partial_name, @dataset_url) do |_stdin, _stdout, _stderr, wait_thr|
      unless wait_thr.value == 0
        raise "Downloading exited with status #{wait_thr.value.inspect}"
      end
    end

    unless local_bytes == remote_bytes
      raise "Only downloaded #{local_bytes} bytes, expected to download #{remote_bytes}"
    end

    File.rename(@partial_name, @output_path)

    puts 'Download complete'
  end

  # gets the size, in bytes, from the Content-Length header of the remote
  def remote_bytes

    return @remote_bytes if @remote_bytes.present?

    Open3.popen3('curl', '-I', @dataset_url) do |_stdin, stdout, _stderr, _wait_thr|
      lines = stdout.read.split("\n")

      lines.each do |line|

        next unless line.start_with? 'Content-Length:'

        @remote_bytes = line.split(': ').last.to_i

        return @remote_bytes
      end
    end

    raise 'No Content-Length header'
  end

  # gets the size, in bytes, of the downloaded file
  def local_bytes
    File.size(@partial_name)
  end

end
