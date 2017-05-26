
module GribConvert
  LEVELS = [1, 2, 3, 5, 7, 10, 20, 30, 50, 70, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 925, 950, 975, 1000]

  class << self
    def convert_level(path, level)
      new_file = "#{path.split('.').first}_l#{level}.gribp"
      return if File.exist?(new_file) && File.size(new_file) > 10000

      command = "grib_get_data -p shortName -w level=#{level} #{path}"

      File.open(new_file, 'wb') do |file|
        IO.popen(command) do |io|
          io.gets # skip first line
          while (line = io.gets) do
            lat, lon, value, label = line.split ' '
            next unless label == 'u' || label == 'v'
            next if lat == '' || lon == '' || value == ''

            file.write "#{[lat.to_f].pack('g')}#{[lon.to_f].pack('g')}#{[value.to_f].pack('g')}#{label[0]}"
          end
        end
      end
    end

    def convert(path)
      true_start = Time.now

      # Note: multithreading this does not help as it must do IO on the same input files
      LEVELS.reverse_each do |level|
        start = Time.now
        convert_level(path, level)
        puts "Converted #{level} (#{(Time.now - start).round(2)}s)"
      end

      puts "Converted #{LEVELS.length} levels (#{(Time.now - true_start).round(2)}s)"
    end
  end
end