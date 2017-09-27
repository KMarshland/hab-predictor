require 'fileutils'

class DownloadWorker

  # Downloads the given dataset from the remote
  def perform(dataset_url)

    @dataset_url = dataset_url

    @filename = @dataset_url.split('/').pop

    filestem = filename.split('.').first
    @partial_name = "#{filestem}.grb2.partial"

    command = "curl -o #{@partial_name}"

    unless local_bytes == remote_bytes
      return
    end

    File.rename(@partial_name, @filename)
  end

  # gets the size, in bytes, from the Content-Length header of the remote
  def remote_bytes
    command = "curl -I #{@dataset_url}"
    output = `#{command}`
  end

  # gets the size, in bytes, of the downloaded file
  def local_bytes

  end

end
