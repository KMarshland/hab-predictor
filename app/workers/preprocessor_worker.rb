require 'fileutils'

class PreprocessorWorker
  LEVELS = [1, 2, 3, 5, 7, 10, 20, 30, 50, 70, 80, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 925, 950, 975, 1000]
  CELL_SIZE = 25 # lower means the predictor runs faster, higher means this thing does
  MEMORY_BUFFER_SIZE = 100_000 # in units of velocity tuple count (approx 20 bytes each)
  APPROX_TUPLE_COUNT = 10_500_000


  def perform(dataset_url)

    filename = dataset_url.split('/').pop
    path = Rails.root.join('data', filename)

    puts "Preprocessing #{filename}"

    true_start = Time.now

    base_dir = path.to_s.split('.').first

    FileUtils::mkdir_p base_dir

    LEVELS.each.with_index do |level, i|
      level_start = Time.now

      read_to_format(dir: base_dir, path: path, level: level)

      seconds = Time.now - level_start
      total_seconds = Time.now - true_start

      percentage = (100.0 * (i+1)/LEVELS.count)
      extrapolation = total_seconds / (percentage / 100.0) - total_seconds
      percentage_string = "#{percentage.round(2)}%".ljust(6)

      puts "\t Level #{level.to_s.rjust(4)} written (#{percentage_string} complete; #{seconds.round}s; ~#{extrapolation.round}s remaining)"
    end

    seconds = Time.now - true_start
    puts "-> Converted #{LEVELS.length} levels (#{seconds.round(2)}s, #{(seconds / LEVELS.length.to_f).round(2)}s avg)"
    puts
  end

  def read_to_format(path:, dir:, level:)
    # each buffer tuple is an array [level, lat, lon, u, v]
    incomplete_buffer = {}
    complete_buffer = []

    # flushes existing (cost: ~50s)
    written = 0
    flush_buffer = -> {
      file_contents = {}

      complete_buffer.each_with_index do |atmo, index|
        next if atmo.nil?
        next if atmo[2].nil? || atmo[3].nil? || atmo[4].nil?

        grid_lat = (atmo[0] / CELL_SIZE).floor * CELL_SIZE
        grid_lon = (atmo[1] / CELL_SIZE).floor * CELL_SIZE

        filename = "L#{level}/C#{grid_lat}_#{grid_lon}.gribp"
        file = (file_contents[filename] ||= '')

        # convert the buffer to strings
        file << "#{[atmo[0]].pack('g')}#{[atmo[1]].pack('g')}#{[atmo[2]].pack('g')}#{[atmo[3]].pack('g')}#{[atmo[4]].pack('g')}"

        # don't rewrite the same data
        complete_buffer[index] = nil
        written += 1
      end

      # write the buffer to files
      file_contents.each do |name, contents|

        FileUtils::mkdir_p "#{dir}/#{name.match(/L\d+/).to_s}"

        file = File.open("#{dir}/#{name}", 'ab')
        file.write contents

        file.close
      end

      complete_buffer = []
    }

    # start parsing (cost: ~20s)

    command = "grib_get_data -p shortName -w shortName=u/v/t,level=#{level} #{path}"

    IO.popen(command) do |io|
      io.gets # skip first line

      # interpret (cost: ~120s)
      while (line = io.gets) do
        lat, lon, value, label = line.split ' '
        next if lat == '' || lon == '' || value == ''

        lat = lat.to_f
        lon = lon.to_f

        key = :"#{lat}_#{lon}"
        buffer = incomplete_buffer[key]

        # weird if structure but microoptimizations actually matter here

        if buffer
          if label == 'u'
            buffer[2] = value.to_f
          elsif label == 'v'
            buffer[3] = value.to_f
          else
            buffer[4] = value.to_f
          end

          # order is because grib reads in order t, u, v, so this will short circuit the fastest
          complete_buffer << incomplete_buffer.delete(key) unless buffer[3].nil? || buffer[2].nil? || buffer[4].nil?
        else
          is_u = label == 'u'
          is_v = !is_u && label == 'v'
          is_t = !is_u && !is_v

          incomplete_buffer[key] = [
              lat,
              lon,
              is_u ? value.to_f : nil,
              is_v ? value.to_f : nil,
              is_t ? value.to_f : nil
          ]
        end

        flush_buffer[] if complete_buffer.size >= MEMORY_BUFFER_SIZE

      end
    end

    flush_buffer[]
  end

end