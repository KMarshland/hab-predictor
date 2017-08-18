class StatusController < ApplicationController

  def status
    render json: {
        up: true,
        datasets: Predictor.datasets
    }
  end

end
