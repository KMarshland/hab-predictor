require 'fileutils'

module GribConvert
  LEVELS = [1, 2, 3, 5, 7, 10, 20, 30, 50, 70, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 925, 950, 975, 1000]
  CELL_SIZE = 25 # we don't want to have too many open files. Limit defaults to 256
  BUFFER_SIZE = 4096 # in units of bytes

  class << self
    def convert_level(level:, path:, dir:)
      base_dir = "#{dir}/L#{level}"
      FileUtils::mkdir_p base_dir

      return if File.exist? "#{base_dir}/.done" # don't reparse

      # pre-open files
      files = {}
      buffers = {}

      (-90..90).step(CELL_SIZE).each do |lat|
        grid_lat = (lat / CELL_SIZE).floor * CELL_SIZE

        files[grid_lat] ||= {}
        buffers[grid_lat] ||= {}

        (0..360).step(CELL_SIZE).each do |lon|
          grid_lon = (lon / CELL_SIZE).floor * CELL_SIZE
          files[grid_lat][grid_lon] ||= File.open("#{base_dir}/C#{grid_lat}_#{grid_lon}.gribp", 'wb')
          buffers[grid_lat][grid_lon] ||= ''
        end
      end

      # start parsing
      command = "grib_get_data -p shortName -w level=#{level} #{path}"

      IO.popen(command) do |io|
        io.gets # skip first line
        while (line = io.gets) do
          lat, lon, value, label = line.split ' '
          next unless label == 'u' || label == 'v'
          next if lat == '' || lon == '' || value == ''

          lat = lat.to_f
          lon = lon.to_f

          grid_lat = (lat / CELL_SIZE).floor * CELL_SIZE
          grid_lon = (lon / CELL_SIZE).floor * CELL_SIZE

          buffer = buffers[grid_lat][grid_lon]
          buffer << "#{[lat].pack('g')}#{[lon].pack('g')}#{[value.to_f].pack('g')}#{label[0]}"

          if buffer.length > BUFFER_SIZE
            files[grid_lat][grid_lon].write buffer
            buffers[grid_lat][grid_lon] = ''
          end

        end
      end

      files.each do |_, file_list|
        file_list.each do |_, file|
          file.close
        end
      end

      FileUtils::touch "#{base_dir}/.done"
    end

    def convert(path)
      puts "Converting #{path}"

      true_start = Time.now

      base_dir = path.split('.').first

      FileUtils::mkdir_p base_dir

      # Note: multithreading this does not help as it must do IO on the same input files
      LEVELS.reverse_each do |level|
        start = Time.now

        convert_level(path: path, level: level, dir: base_dir)

        puts "    converted L#{level} (#{(Time.now - start).round(2)}s)"
      end

      seconds = Time.now - true_start
      puts "-> Converted #{LEVELS.length} levels (#{seconds.round(2)}s, #{(seconds / LEVELS.length.to_f).round(2)}s avg)"
      puts
    end

    def convert_folder(dir, serial:true)
      threads = []
      Dir.entries(dir).each do |entry|
        next unless entry =~ /\.grb2/ # only try to parse grib files

        if serial
          convert "#{dir}/#{entry}"
        else
          threads << Thread.new do
            convert "#{dir}/#{entry}"
          end
        end

      end

      threads.map(&:join) unless serial
    end
  end
end