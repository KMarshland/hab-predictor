require_relative 'processed_datasets'


iteration = 0
on_date = nil

loop do

  # figure out which date you should look for once every 30 iterations
  if iteration % 30 == 0
    on_date = ProcessedDatasets.last_date
  end

  dataset = ProcessedDatasets.on on_date

  puts "\t [dataset downloader] #{dataset.size} items in #{on_date.strftime('%Y%m%d')}"

  dataset.each do |item|
    ImportWorker.new.perform item, overwrite: false
  end

  sleep 1.minute

end