require Rails.root.join('lib', 'storage', 'processed_datasets.rb')

class StatusController < ApplicationController

  def status
    render json: {
        up: true
    }
  end

  def datasets
    @downloaded = Predictor.datasets
    @processed = ProcessedDatasets.last_dataset

    respond_to do |format|

      format.json {
        render json: {
            downloaded: @downloaded,
            processed: @processed
        }
      }

      format.html {
        @intersection = @downloaded & @processed

        @not_downloaded = @processed - @downloaded
        @not_processed = @downloaded - @processed

        render
      }

    end
  end

end
