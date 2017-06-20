module Prediction

  class << self

    def datasets
      sets = []
      # Folder names are of the form yyyymmdd
      Dir.entries(Rails.root.join('lib', 'data')).each do |filename|
        num_date = File.basename(filename).to_i

        sets << DateTime.strptime(num_date.to_s, '%Y%m%d') if num_date > 0
      end

      sets.sort
    end

    def latest_dataset
      datasets().last
    end

  end
end

require_relative './active_guidance'
