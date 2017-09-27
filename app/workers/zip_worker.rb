require 'zip'

class ZipWorker

  def perform(dataset_url)
    folder_name = dataset_url.split('/').last.split('.').first
    puts "Zipping #{folder_name}"

    output_file = Rails.root.join('data', "#{folder_name}.zip")

    File.delete(output_file) if File.exists? output_file

    @input_dir = Rails.root.join('data', folder_name)
    @count = 0

    entries = Dir.entries(@input_dir) - %w(. ..)
    Zip::File.open(output_file, ::Zip::File::CREATE) do |io|
      write_entries entries, '', io

      puts ''
      puts "#{@count} files zipped"
    end
  end

  private

  # A helper method to make the recursion work.
  def write_entries(entries, path, io)
    entries.each do |e|
      zip_file_path = path == '' ? e : File.join(path, e)
      disk_file_path = File.join(@input_dir, zip_file_path)

      @count += 1
      print '.'

      if File.directory? disk_file_path
        recursively_deflate_directory(disk_file_path, io, zip_file_path)
      else
        put_into_archive(disk_file_path, io, zip_file_path)
      end
    end
  end

  def recursively_deflate_directory(disk_file_path, io, zip_file_path)
    io.mkdir zip_file_path
    subdir = Dir.entries(disk_file_path) - %w(. ..)
    write_entries subdir, zip_file_path, io
  end

  def put_into_archive(disk_file_path, io, zip_file_path)
    io.get_output_stream(zip_file_path) do |f|
      f.write(File.open(disk_file_path, 'rb').read)
    end
  end

end
