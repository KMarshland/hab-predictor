
path = ARGV.last
levels = [1, 2, 3, 5, 7, 10, 20, 30, 50, 70, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 925, 950, 975, 1000]
# levels = [500]

def convert_level(path, level)
  new_file = "#{path.split('.').first}_l#{level}.gribp"
  return if File.exist? new_file

  command = "grib_get_data -p shortName -w level=#{level} #{path}"

  File.open(new_file, 'wb') do |file|
    IO.popen(command) do |io|
      io.gets # skip first line
      while (line = io.gets) do
        lat, lon, value, label = line.split ' '
        next unless label == 'u' || label == 'v'

        file.write "#{[lat, lon, value].map(&:to_f).pack('g')}#{label}\n"
      end
    end
  end
end

true_start = Time.now

# Note: multithreading this does not help as it must do IO on the same input files
levels.reverse_each do |level|
  start = Time.now
  convert_level(path, level)
  puts "Converted #{level} (#{(Time.now - start).round(2)}s)"
end

puts "Converted #{levels.length} levels (#{(Time.now - true_start).round(2)}s)"