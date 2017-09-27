require 'fileutils'

class PreprocessorWorker
  LEVELS = [1, 2, 3, 5, 7, 10, 20, 30, 50, 70, 80, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 925, 950, 975, 1000]
  CELL_SIZE = 25 # lower means the predictor runs faster, higher means this thing does
  MEMORY_BUFFER_SIZE = 100_000 # in units of velocity tuple count (approx 20 bytes each)
  APPROX_TUPLE_COUNT = 10_500_000


  def perform(path)
    puts "Converting #{path}"

    true_start = Time.now

    base_dir = path.split('.').first

    FileUtils::mkdir_p base_dir

    read_to_format(dir: base_dir, path: path)

    seconds = Time.now - true_start
    puts "-> Converted #{LEVELS.length} levels (#{seconds.round(2)}s, #{(seconds / LEVELS.length.to_f).round(2)}s avg)"
    puts
  end

  def read_to_format(path:, dir:)
    return if File.exist? "#{dir}/.done" # don't reparse

    # TODO: clear old files

    # each buffer tuple is an array [level, lat, lon, u, v]
    incomplete_buffer = {}
    complete_buffer = []

    # flushes existing (cost: ~50s)
    written = 0
    flush_buffer = -> {
      file_contents = {}

      complete_buffer.each_with_index do |vel, index|
        next if vel.nil?
        next if vel[3].nil? || vel[4].nil?

        grid_lat = (vel[1] / CELL_SIZE).floor * CELL_SIZE
        grid_lon = (vel[2] / CELL_SIZE).floor * CELL_SIZE

        filename = "L#{vel[0]}/C#{grid_lat}_#{grid_lon}.gribp"
        file = (file_contents[filename] ||= '')

        # convert the buffer to strings
        file << "#{[vel[1]].pack('g')}#{[vel[2]].pack('g')}#{[vel[3]].pack('g')}#{[vel[4]].pack('g')}"

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
      puts "\t #{written} tuples written (~#{(100.0*written/APPROX_TUPLE_COUNT.to_f).round(2)}%)" if written > 0
    }

    # start parsing (cost: ~20s)
    command = "grib_get_data -p shortName,level -w shortName=u/v,level!=0 #{path}"

    IO.popen(command) do |io|
      io.gets # skip first line

      # interpret (cost: ~120s)
      while (line = io.gets) do
        lat, lon, value, label, level = line.split ' '
        next if lat == '' || lon == '' || value == ''

        lat = lat.to_f
        lon = lon.to_f

        key = :"#{level}_#{lat}_#{lon}"
        buffer = incomplete_buffer[key]

        # weird if structure but microoptimizations actually matter here

        if buffer
          if label == 'u'
            buffer[3] = value.to_f
          else
            buffer[4] = value.to_f
          end

          complete_buffer << incomplete_buffer.delete(key)
        else
          is_u = label == 'u'
          incomplete_buffer[key] = [
              level,
              lat,
              lon,
              is_u ? value.to_f : nil,
              is_u ? nil : value.to_f,
          ]
        end

        flush_buffer[] if complete_buffer.size >= MEMORY_BUFFER_SIZE
      end

    end

    flush_buffer[]

    FileUtils::touch "#{dir}/.done"
  end

end